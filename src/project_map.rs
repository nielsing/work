use std::collections::HashMap;

use serde_json;

use crate::arguments::TimeFormat;
use crate::log_file::Event;
use crate::time::format_time;

/// These constants are used to add clarity to the `add_events` function for the ProjectMap.
const START: usize = 0;
const STOP: usize = 1;

/// ProjectMap maps projects to descriptions which in turn is mapped to total spent time.
///
/// A project is mapped to a map which maps descriptions to the total time spent on a given project
/// with a given description.
pub type ProjectMap = HashMap<String, HashMap<String, i64>>;

pub trait ProjectMapMethods {
    // Functions for insertion.
    fn add_event(&mut self, time: &i64, event: &Event);
    fn add_events(&mut self, events: &[(i64, Event)]);
    fn add_clean_event(&mut self, time: &i64, event: &Event);

    // Functions for output.
    fn as_csv(&self, time_format: &TimeFormat) -> String;
    fn as_json(&self, time_format: &TimeFormat) -> String;
}

impl ProjectMapMethods for ProjectMap {
    /// Adds a singular event and the time spent on it to the ProjectMap.
    fn add_event(&mut self, time: &i64, event: &Event) {
        self.entry(event.to_project())
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
    }

    /// Adds multiple events to the ProjectMap. This function receives a list of events assumed to
    /// be in [START, STOP, START, STOP] order and inserts them into the ProjectMap.
    fn add_events(&mut self, events: &[(i64, Event)]) {
        events.chunks(2).for_each(|pair| {
            let time = pair[STOP].0 - pair[START].0;
            self.add_event(&time, &pair[START].1);
        });
    }

    /// Assumes the given project does not exist within the ProjectMap and blindly inserts it.
    fn add_clean_event(&mut self, time: &i64, event: &Event) {
        self.insert(event.to_project(), {
            let mut new = HashMap::new();
            new.insert(event.to_description(), *time);
            new
        });
    }

    /// Returns a CSV format of the ProjectMap as a string.
    fn as_csv(&self, time_format: &TimeFormat) -> String {
        let mut csv = String::from("Project,Description,Time Spent\n");
        self.iter().for_each(|(project, descs)| {
            descs.iter().for_each(|(desc, time)| {
                csv.push_str(&format!(
                    "{},{},{}\n",
                    project,
                    desc,
                    format_time(time_format, *time)
                ));
            });
        });
        csv
    }

    /// Returns a JSON format of the ProjectMap as a string.
    fn as_json(&self, time_format: &TimeFormat) -> String {
        // This is incredibly dirty code, I know. I just can't be bothered with implementing a
        // custom serde serializer right now and this works ok.
        let mut tmp_map = HashMap::new();
        for (project, descs) in self {
            let mut tmp_descs = HashMap::new();
            for (desc, time) in descs {
                tmp_descs.insert(desc, format_time(time_format, *time));
            }
            tmp_map.insert(project, tmp_descs);
        }
        serde_json::to_string_pretty(&tmp_map).unwrap()
    }
}
