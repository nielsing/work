use structopt::StructOpt;

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
        /// Set time format to total number of minutes
        #[structopt(long)]
        minutes: bool,
        /// Set time format to approximate number of minutes
        #[structopt(short, long)]
        minutes_approx: bool,
        /// Set time format to  approximate number of hours
        #[structopt(short, long)]
        hours_approx: bool,
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
}
