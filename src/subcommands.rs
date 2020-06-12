use std::env;
use std::process::Command;

use crate::arguments::TimeFormat;
use crate::error::{AppError, ErrorKind};
use crate::log_file::*;
use crate::project_map::ProjectMapMethods;
use crate::time;

// Helper function to simplify checks of a given Event.
// Returns false if the last log states that no work is in progress, true otherwise.
//
// Mainly used to make the `start` function easier on the eyes.
fn is_working(event: &Event) -> bool {
    match event {
        Event::Stop(_, _) => false,
        Event::Start(_, _) => true,
    }
}

/// The `start` function corresponds to the `start` command.
///
/// The function reads the log for the last event and makes sure that the user isn't starting new
/// work while other work is in progress. This is done because one should only be working on a
/// single thing at a time.
///
/// If the user isn't trying to append a double `start` event, the function appends a `start` event
/// to the log.
pub fn start(
    log: &mut LogFile,
    project: Option<String>,
    description: Option<String>,
) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;
    if is_working(&event) {
        return Err(AppError::new(ErrorKind::User(
            "Please stop the current work before starting new work.".to_string(),
        )));
    }
    log.append_event_now(&Event::Start(project, description))?;
    Ok(0)
}

/// The `stop` function corresponds to the `stop` command.
///
/// The function reads the log for the last event and makes sure the user isn't trying to stop
/// already stopped work.
///
/// If the last event was a `start` event the function appends a `stop` event to the log with the
/// same project description as the final `start` event in the log. This is done to make life
/// easier when adding up time spent on projects in the `log_file.rs`.
pub fn stop(log: &mut LogFile) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;

    match &event {
        Event::Stop(_, _) => {
            return Err(AppError::new(ErrorKind::User(
                "Unable to stop, no work in progress!".to_string(),
            )))
        }
        Event::Start(None, None) => log.append_event_now(&Event::Stop(None, None))?,
        Event::Start(Some(project), None) => {
            log.append_event_now(&Event::Stop(Some(project.to_owned()), None))?
        }
        Event::Start(None, Some(description)) => {
            log.append_event_now(&Event::Stop(None, Some(description.to_owned())))?
        }
        Event::Start(Some(project), Some(description)) => log.append_event_now(&Event::Stop(
            Some(project.to_owned()),
            Some(description.to_owned()),
        ))?,
    }
    Ok(0)
}

/// The `status` function corresponds to the `status` command.
///
/// The function outputs the final event in the log in human readable form. That is, the function
/// outputs "Free" if the final event is a `stop` event, "Working" if the final event is a `start`
/// event with no project, and "Working on [PROJECT_NAME]" if the final event is a `start` event
/// with a project name.
pub fn status(log: &mut LogFile) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;
    match event {
        Event::Stop(_, _) => println!("Free"),
        Event::Start(None, _) => println!("Working"),
        Event::Start(Some(project), _) => println!("Working on {}", project),
    }
    Ok(0)
}

/// The `working_or_free` function corresponds to both the `free` and the `working` commands.
///
/// If the command called is `free` the function exits with an exit code of 0 if the final event in
/// the log is a `stop` event, and 1 otherwise.
///
/// If the command called is `working` the function exits with an exit code of 0 if the final event
/// in the log is a `start` event, and 1 otherwise.
pub fn working_or_free(log: &mut LogFile, check_working: bool) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;
    match (event, check_working) {
        // Not working and user questions whether he is free -> Yes
        (Event::Stop(_, _), false) => Ok(0),
        // Not working and user questions whether he is working -> No
        (Event::Stop(_, _), true) => Ok(1),
        // Working and user questions whether he is free -> No
        (Event::Start(_, _), false) => Ok(1),
        // Working and user questions whether he is working -> Yes
        (Event::Start(_, _), true) => Ok(0),
    }
}

/// The `of` function corresponds to the `of` command.
///
/// The function receives the user inputted interval, parses it, finds all work that was done
/// within the given interval, adds the time spent on projects together, and finally outputs the
/// results.
///
/// The user inputted interval can be of the following forms:
/// * X               meaning at X o'clock
/// * X:Y             meaning Y minutes past X o'clock
/// * Xm              meaning X minutes ago
/// * Xh              meaning X hours ago
/// * X:Yh            meaning X hours and Y minutes ago
/// * D X:Y           meaning since day D at Y minutes past X o'clock
/// * D-M X:Y         meaning since day D and month M at Y minutes past X o'clock
/// * today           means last possible midnight
/// * yesterday       means midnight of yesterday
/// * [START] - [END] means anything between START and END (inclusive) where START and END are any
/// of the forms above.
///
/// Some of these inputs can be ambiguous, if an input given is ambiguous the last possible time
/// will be chosen.
///
/// The maximum of the two values (START and END) in an interval is interpreted as the end date.
pub fn of(
    log: &mut LogFile,
    interval_input: &str,
    csv: bool,
    json: bool,
    time_format: TimeFormat,
) -> Result<i32, AppError> {
    let mut interval = time::Interval::try_parse(interval_input, &time::Search::Backward)?;

    if interval_input == "yesterday" {
        interval.end = time::today_date_time().timestamp();
    }

    let project_times = log.tally_time(&interval)?;
    if let Some(map) = project_times {
        if csv {
            println!("{}", map.as_csv(&time_format));
        } else if json {
            println!("{}", map.as_json(&time_format));
        } else {
            map.iter().for_each(|(key, val)| {
                println!(
                    "{} => {}",
                    key.to_string(),
                    time::format_time(&time_format, val.values().sum())
                )
            });
        }
    } else {
        println!("No work done!");
        return Ok(1);
    }
    Ok(0)
}

/// The `since` function corresponds to the `since` command.
///
/// The command makes sure that the user is free. If there is no work in progress, the command will
/// append a `start` event with `project` name and `description` at the specified time and a `stop`
/// event for the current time.
pub fn since(
    log: &mut LogFile,
    time: &str,
    project: Option<String>,
    description: Option<String>,
    r#continue: bool,
) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;
    if is_working(&event) {
        return Err(AppError::new(ErrorKind::User(
            "Please stop the current work before registering new work.".to_string(),
        )));
    }

    let interval = time::Interval::try_parse(time, &time::Search::Backward)?;
    log.append_event(
        &Event::Start(project.clone(), description.clone()),
        interval.start,
    )?;
    if !r#continue {
        log.append_event_now(&Event::Stop(project, description))?;
    }
    Ok(0)
}

/// The `until` function corresponds to the `until` command.
///
/// The command makes sure that user is free. If there is no work in progress the command will
/// append a `start` event for current time with `project` name and `description` and will finish by
/// appending a `stop` event at the specified time.
pub fn until(
    log: &mut LogFile,
    time: &str,
    project: Option<String>,
    description: Option<String>,
) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;
    if is_working(&event) {
        return Err(AppError::new(ErrorKind::User(
            "Please stop the current work before starting new work.".to_string(),
        )));
    }

    let interval = time::Interval::try_parse(time, &time::Search::Forward)?;
    log.append_event_now(&Event::Start(project.clone(), description.clone()))?;
    log.append_event(&Event::Stop(project, description), interval.end)?;
    Ok(0)
}

/// The `between` function corresponds to the `between` command.
///
/// The command makes sure that user is free. If there is no work in progress the command will
/// append a `start` event at the specified start time with `project` name and `description` and
/// will finish by appending a `stop` event at the specified end time.
pub fn between(
    log: &mut LogFile,
    time: &str,
    project: Option<String>,
    description: Option<String>,
) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;
    if is_working(&event) {
        return Err(AppError::new(ErrorKind::User(
            "Please stop the current work before starting new work.".to_string(),
        )));
    }

    let interval = time::Interval::try_parse(time, &time::Search::Backward)?;
    log.append_event(
        &Event::Start(project.clone(), description.clone()),
        interval.start,
    )?;
    log.append_event(&Event::Stop(project, description), interval.end)?;
    Ok(0)
}

/// The `while` function corresponds to the `while` command.
///
/// The command executes a given command tagged with the project name and description.
/// This is done by searching for the `SHELL` environment variable and then executing that shell
/// with the `-c` flag with the user inputted command appended to the back.
///
/// This will probably not work for windows machines or darwin/linux users who use a niche shell.
/// If windows support is requested it is possible to add a windows compiler flag to handle that
/// cause. Possibly by spawning powershell?
pub fn r#while(
    log: &mut LogFile,
    cmd: &str,
    project: Option<String>,
    description: Option<String>,
) -> Result<i32, AppError> {
    let event = log.get_latest_event()?;
    if is_working(&event) {
        return Err(AppError::new(ErrorKind::User(
            "Please stop the current work before starting new work.".to_string(),
        )));
    }

    let shell = match env::var("SHELL") {
        Ok(name) => name,
        Err(_) => "sh".to_string(),
    };

    let cmd: Vec<&str> = cmd.split_whitespace().collect();
    match Command::new(&shell).arg("-c").args(&cmd).spawn() {
        Ok(mut child) => {
            log.append_event_now(&Event::Start(project.clone(), description.clone()))?;
            let status = match child.wait() {
                Ok(status) => status,
                Err(e) => {
                    return Err(AppError::new(ErrorKind::System(format!(
                        "Process failed to start: {}",
                        e
                    ))));
                }
            };
            log.append_event_now(&Event::Stop(project, description))?;
            if status.success() {
                return Ok(0);
            } else {
                return Err(AppError::new(ErrorKind::System(
                    "Process failed to execute".to_string(),
                )));
            }
        }
        Err(e) => {
            return Err(AppError::new(ErrorKind::System(format!(
                "Failed to start {}: {}",
                &shell, e
            ))));
        }
    }
}
