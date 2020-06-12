use structopt::StructOpt;

use work::arguments::*;
use work::error::{AppError, ErrorKind};
use work::log_file::*;
use work::subcommands::*;

fn main() {
    let args = Args::from_args();
    std::process::exit(match run_app(args) {
        // If we get back an Ok it can be an error code of either 0 or 1.
        // This is because of the  `of`, `working`, and `free` commands.
        Ok(val) => val,
        Err(err) => match &err.kind() {
            ErrorKind::User(msg) => {
                eprintln!("{}", msg);
                2
            }
            ErrorKind::LogFile(msg) => {
                eprintln!("{}", msg);
                3
            }
            ErrorKind::System(msg) => {
                eprintln!("{}", msg);
                4
            }
        },
    });
}

fn run_app(args: Args) -> Result<i32, AppError> {
    let mut log = LogFile::new()?;

    match args.subcommand {
        SubCommand::Start {
            project,
            description,
        } => start(&mut log, project, description),
        SubCommand::Stop => stop(&mut log),
        SubCommand::Status => status(&mut log),
        SubCommand::Free => working_or_free(&mut log, false),
        SubCommand::Working => working_or_free(&mut log, true),
        SubCommand::Of {
            interval,
            csv,
            json,
            time_format,
        } => of(&mut log, &interval, csv, json, time_format),
        SubCommand::Since {
            time,
            project,
            description,
            r#continue,
        } => since(&mut log, &time, project, description, r#continue),
        SubCommand::Until {
            time,
            project,
            description,
        } => until(&mut log, &time, project, description),
        SubCommand::While {
            cmd,
            project,
            description,
        } => r#while(&mut log, &cmd, project, description),
    }
}
