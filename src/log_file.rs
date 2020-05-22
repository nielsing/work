use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;

use dirs;

use crate::error::{AppError, ErrorKind};
use crate::time;

/// These constants are used to add clarity to the `add_to_hashmap` closure in the `tally_time`
/// function.
const START: usize = 0;
const STOP: usize = 1;

/// The `Event` enum describes a single event in the log. Each event in the log can either be a
/// `start` event with or without a project description or a `stop` event with or without a project
/// description.
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Event {
    Start(Option<String>, Option<String>),
    Stop(Option<String>, Option<String>),
}

impl Event {
    fn to_project(&self) -> String {
        match self {
            Event::Stop(None, _) => "Unnamed project".to_string(),
            Event::Start(None, _) => "Unnamed project".to_string(),
            Event::Stop(Some(project), _) => project.to_string(),
            Event::Start(Some(project), _) => project.to_string(),
        }
    }

    fn to_description(&self) -> String {
        match self {
            Event::Stop(_, None) => "No description".to_string(),
            Event::Start(_, None) => "No description".to_string(),
            Event::Stop(_, Some(description)) => description.to_string(),
            Event::Start(_, Some(description)) => description.to_string(),
        }
    }
}

// For nice outputting of an Event type.
impl ToString for Event {
    fn to_string(&self) -> String {
        match self {
            Event::Stop(None, None) => "Unnamed project".to_string(),
            Event::Start(None, None) => "Unnamed project".to_string(),
            Event::Stop(None, Some(description)) => format!("Unnamed project - {}", description),
            Event::Start(None, Some(description)) => format!("Unnamed project - {}", description),
            Event::Stop(Some(project), None) => project.to_string(),
            Event::Start(Some(project), None) => project.to_string(),
            Event::Stop(Some(project), Some(description)) => {
                format!("{} - {}", project, description)
            }
            Event::Start(Some(project), Some(description)) => {
                format!("{} - {}", project, description)
            }
        }
    }
}

// Used for parsing Events out of the log.
impl From<&str> for Event {
    fn from(event: &str) -> Self {
        let values: Vec<&str> = event.split(',').map(|s| s.trim()).collect();
        match &values[..] {
            [_, "Stop", "", ""] => Event::Stop(None, None),
            [_, "Start", "", ""] => Event::Start(None, None),
            [_, "Start", project, ""] => Event::Start(Some(project.to_string()), None),
            [_, "Stop", project, ""] => Event::Stop(Some(project.to_string()), None),
            [_, "Start", project, description] => {
                Event::Start(Some(project.to_string()), Some(description.to_string()))
            }
            [_, "Stop", project, description] => {
                Event::Stop(Some(project.to_string()), Some(description.to_string()))
            }
            _ => Event::Stop(None, None),
        }
    }
}

/// The `LogFile` struct is a wrapper around a `File`.
///
/// This ensures that one can only do "logging" actions to the log file. That is one can only
/// append to the file or read from it. The `LogFile` also implements some handy functions for
/// dealing with the log, like appending events or fetching the latest event of a log file.
pub struct LogFile {
    log: File,
}

impl LogFile {
    /// Fetches the default path for the log file, creates it if it doesn't exist and finally sets
    /// the `log` to the open file descriptor of the log file.
    ///
    /// If any of these actions fail to finish, the function will return an error message.
    pub fn new() -> Result<Self, AppError> {
        let file_path = Self::log_file_path()?;
        Self::create_path(&file_path)?;

        Ok(LogFile {
            log: match OpenOptions::new()
                .append(true)
                .create(true)
                .read(true)
                .open(file_path)
            {
                Ok(file) => file,
                Err(e) => {
                    return Err(AppError::from(e));
                }
            },
        })
    }

    /// Appends a given `Event` to the log with the given `timestamp`.
    /// If it fails to append to the log, the function returns an error message.
    pub fn append_event(&mut self, event: &Event, timestamp: i64) -> Result<(), AppError> {
        match event {
            Event::Start(Some(project), Some(description)) => {
                self.write(&format!("{},Start,{},{}", timestamp, project, description))?
            }
            Event::Stop(Some(project), Some(description)) => {
                self.write(&format!("{},Stop,{},{}", timestamp, project, description))?
            }
            Event::Start(Some(project), None) => {
                self.write(&format!("{},Start,{},", timestamp, project))?
            }
            Event::Stop(Some(project), None) => {
                self.write(&format!("{},Stop,{},", timestamp, project))?
            }
            Event::Start(None, Some(description)) => {
                self.write(&format!("{},Start,,{}", timestamp, description))?
            }
            Event::Stop(None, Some(description)) => {
                self.write(&format!("{},Stop,,{}", timestamp, description))?
            }
            Event::Start(None, None) => self.write(&format!("{},Start,,", timestamp))?,
            Event::Stop(None, None) => self.write(&format!("{},Stop,,", timestamp))?,
        };
        Ok(())
    }

    /// Appends a given `Event` to the log using the current UNIX timestamp of the system.
    /// If it fails to append to the log, the function returns an error message.
    pub fn append_event_now(&mut self, event: &Event) -> Result<(), AppError> {
        self.append_event(&event, time::now())
    }

    /// Reads the whole log into a `String` and returns the final event in the log.
    /// If it fails to read the log file, the function returns an error message.
    pub fn get_latest_event(&mut self) -> Result<Event, AppError> {
        let mut events = String::new();
        match self.log.read_to_string(&mut events) {
            Ok(_) => {
                let last_event = events.lines().rev().next();
                match last_event {
                    Some(event) => Ok(Event::from(event)),
                    None => Ok(Event::Stop(None, None)),
                }
            }
            Err(e) => Err(AppError::from(e)),
        }
    }

    /// Finds all events that are within a given `Interval` and sums up the time spent on each
    /// project, then it returns the results as a `HashMap`.
    ///
    /// This is done by first filtering the events of the log file for events that contain
    /// timestamps that are within the timestamps of the given interval.
    ///
    /// The filtered events returned can be lists in the following forms:
    /// * An empty list.
    /// * List containing a single `Stop` or `Start` event.
    /// * List containing more than one event.
    ///     - The first event is a `Start` event and the last event is a `Stop` event.
    ///     - The first event is a `Start` event and the last event is a `Start` event.
    ///     - The first event is a `Stop` event and the last event is a `Stop` event.
    ///     - The first event is a `Stop` event and the last event is a `Start` event.
    ///
    /// The `Start` `Stop` case is the most favourable case to work with as it is the most simple
    /// case. However the other cases can be thought of as an addition to that case.
    ///
    /// For example the `Start` `Start` case is just a `Start` `Stop` case with an added `Start`
    /// event in the end. Thinking of the cases in this matter makes it much simpler to sum the
    /// events.
    pub fn tally_time(
        &mut self,
        interval: &time::Interval,
    ) -> Result<Option<HashMap<String, HashMap<String, i64>>>, AppError> {
        let events = self.filter_events(interval)?;
        let mut projects: HashMap<String, HashMap<String, i64>> = HashMap::new();

        // Closure for adding a singular event to projects hashmap
        let mut add_event_to_hashmap = |time: &i64, event: &Event| {
            projects
                .entry(event.to_project())
                .and_modify(|map| {
                    map.entry(event.to_description())
                        .and_modify(|x| *x += *time)
                        .or_insert(*time);
                })
                .or_insert({
                    let mut new = HashMap::new();
                    new.insert(event.to_description(), *time);
                    new
                });
        };

        // Closure for adding list of  [start, .., stop] events to projects hashmap
        let add_events_to_hashmap = |events: &[(i64, Event)]| {
            let time = events[STOP].0 - events[START].0;
            add_event_to_hashmap(&time, &events[START].1);
        };

        match &events[..] {
            // Empty list, no entries are within the given interval
            [] => Ok(None),
            // A single stop event
            [(stop_time, event @ Event::Stop(_, _))] => {
                let time = stop_time - interval.start;
                projects.insert(event.to_project(), {
                    let mut new = HashMap::new();
                    new.insert(event.to_description(), time);
                    new
                });
                Ok(Some(projects))
            }
            // A single start event
            [(start_time, event @ Event::Start(_, _))] => {
                let time = interval.end - start_time;
                projects.insert(event.to_project(), {
                    let mut new = HashMap::new();
                    new.insert(event.to_description(), time);
                    new
                });
                Ok(Some(projects))
            }
            // Handling of [start, ..., stop] case
            [(_, Event::Start(_, _)), .., (_, Event::Stop(_, _))] => {
                events.chunks(2).for_each(add_events_to_hashmap);
                Ok(Some(projects))
            }
            // Handling of [start, ..., start] case => [start, ..., stop] + [start]
            [(_, Event::Start(_, _)), .., (start_time, start_event @ Event::Start(_, _))] => {
                events[..events.len() - 1]
                    .chunks(2)
                    .for_each(add_events_to_hashmap);

                // Add extra `start` case
                let time = interval.end - start_time;
                add_event_to_hashmap(&time, &start_event);
                Ok(Some(projects))
            }
            // Handling of [stop, ..., stop] case => [stop] + [start, ..., stop]
            [(stop_time, stop_event @ Event::Stop(_, _)), .., (_, Event::Stop(_, _))] => {
                events[1..].chunks(2).for_each(add_events_to_hashmap);

                // Add extra `stop` case
                let time = stop_time - interval.start;
                add_event_to_hashmap(&time, &stop_event);
                Ok(Some(projects))
            }
            // Handling of [stop, ..., start] case => [stop] + [start, ..., stop] + [start]
            [(stop_time, stop_event @ Event::Stop(_, _)), .., (start_time, start_event @ Event::Start(_, _))] =>
            {
                events[1..events.len() - 1]
                    .chunks(2)
                    .for_each(add_events_to_hashmap);

                // Add extra `stop` and `start` case.
                let extra_stop = stop_time - interval.start;
                let extra_start = interval.end - start_time;
                add_event_to_hashmap(&extra_stop, stop_event);
                add_event_to_hashmap(&extra_start, start_event);
                Ok(Some(projects))
            }
        }
    }

    /// Reads the whole log into a string, parses and filters for the events of the log that
    /// contain a timestamp that is within the given interval (inclusive).
    ///
    /// If it fails to read the log the function returns an error message.
    fn filter_events(&mut self, interval: &time::Interval) -> Result<Vec<(i64, Event)>, AppError> {
        let mut all_events = String::new();
        self.log.read_to_string(&mut all_events)?;

        Ok(all_events
            .lines()
            .map(|line| {
                // Split a line of the log file into two parts: `timestamp` and `Event`.
                // This is done to seperate the timestamp from the rest of data.
                let values: Vec<&str> = line.splitn(2, ',').map(|s| s.trim()).collect();
                // We can call unwrap when parsing the timestamp, since the program should be the
                // only thing interacting with the log file. However a user can corrupt their own
                // log file and make the program panic. This is an accepted risk.
                (values[0].parse::<i64>().unwrap(), Event::from(line))
            })
            .filter(|event| event.0 >= interval.start && event.0 <= interval.end)
            .collect())
    }

    // FIXME: Might need to seek back to start because of append option
    /// Writes a given log event to the log, if it fails to write to the log, the function returns
    /// an error message
    fn write(&mut self, log_event: &str) -> Result<(), AppError> {
        if let Err(e) = writeln!(self.log, "{}", log_event) {
            return Err(AppError::from(e));
        }
        Ok(())
    }

    /// Fetches the path of the `work.log` file. If it fails to find the config folder, the
    /// function returns an error message.
    fn log_file_path() -> Result<PathBuf, AppError> {
        let mut path = match dirs::data_dir() {
            Some(p) => p,
            None => {
                return Err(AppError::new(ErrorKind::LogFile(
                    "Unable to find config folder!".to_string(),
                )));
            }
        };

        path.push("work");
        path.push("work.log");
        Ok(path)
    }

    /// Creates the default path for the `work.log` file if it doesn't exist. If it fails, the
    /// function exits with an error message.
    fn create_path(path: &PathBuf) -> Result<(), AppError> {
        // Can unwrap here because log_file_path should only return [CONFIG_PATH]/work/work.log
        // or [CONFIG_PATH]/work/work.config
        let parent = path.parent().unwrap();
        match create_dir_all(parent) {
            Err(e) => Err(AppError::new(ErrorKind::LogFile(format!(
                "Unable to create 'work' folder: {}",
                e
            )))),
            _ => Ok(()),
        }
    }
}
