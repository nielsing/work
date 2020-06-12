use std::str::FromStr;

use structopt::StructOpt;

use crate::error::{AppError, ErrorKind};

#[derive(StructOpt, Debug)]
#[structopt(name = "Work - Terminal Time Tracker!")]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// Appends a new start event to the log
    #[structopt(alias = "on")]
    Start {
        /// Name of the project
        project: Option<String>,
        /// Description of the given project
        #[structopt(short, long)]
        description: Option<String>,
    },
    /// Appends a new stop event to the log
    Stop,
    /// Prints the status of the last event in the log in human readable form
    Status,
    /// Exits with an error code of 0 if no work is in progress, and 1 otherwise
    Free,
    /// Exits with an error code of 0 if work is in progress, and 1 otherwise
    Working,
    /// Outputs a summary of work done within a given interval
    Of {
        /// The interval to compare start and stop times of work with
        interval: String,
        /// Set output format to CSV
        #[structopt(short, long)]
        csv: bool,
        /// Set output format to JSON
        #[structopt(short, long)]
        json: bool,
        /// Specify the time format of the output
        #[structopt(short, long, possible_values = &["m", "minutes", "ma", "minutes-approx", "h", "hours", "hr", "human-readable"], default_value = "human-readable")]
        time_format: TimeFormat,
    },
    /// Appends a new event to the log that started at a given time
    Since {
        /// Time since work started
        time: String,
        /// Name of the project
        project: Option<String>,
        /// Description of the given project
        #[structopt(short, long)]
        description: Option<String>,
        /// Don't append a stop event to the log
        #[structopt(short, long)]
        r#continue: bool,
    },
    /// Appends an event to the log that stops at a given time
    #[structopt(alias = "for")]
    Until {
        /// Time until work stops
        time: String,
        /// Name of the project
        project: Option<String>,
        /// Description of the given project
        #[structopt(short, long)]
        description: Option<String>,
    },
    /// Appends a start event, executes a given command, and then appends stop event once the
    /// command finishes.
    While {
        /// The command to execute
        cmd: String,
        /// Name of the project
        project: Option<String>,
        /// Description of the given project
        #[structopt(short, long)]
        description: Option<String>,
    },
    Between {
        /// Time interval in which work was done
        time: String,
        /// Name of the project
        project: Option<String>,
        /// Description of the given project
        #[structopt(short, long)]
        description: Option<String>,
    }
}

#[derive(StructOpt, Debug)]
pub enum TimeFormat {
    Minutes,
    MinutesApprox,
    HoursApprox,
    HumanReadable,
}

impl FromStr for TimeFormat {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "m" => Ok(TimeFormat::Minutes),
            "minutes" => Ok(TimeFormat::Minutes),
            "h" => Ok(TimeFormat::HoursApprox),
            "hours" => Ok(TimeFormat::HoursApprox),
            "ma" => Ok(TimeFormat::MinutesApprox),
            "minutes-approx" => Ok(TimeFormat::MinutesApprox),
            "hr" => Ok(TimeFormat::HumanReadable),
            "human-readable" => Ok(TimeFormat::HumanReadable),
            _ => Err(AppError::new(ErrorKind::User(
                "Valid values are [m, minutes, ma, minutes-approx, h, hours, hr, human-readable]"
                    .to_string(),
            ))),
        }
    }
}
