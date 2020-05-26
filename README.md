<h1 align="center">
  <br>
  <a href="https://github.com/nielsing/work"><img src="https://raw.githubusercontent.com/nielsing/work/master/media/work.png" alt="work" width="350px"></a>
  <br>
</h1>

<h1 align="center">Terminal Time Tracker!</h1>

------

## Introduction
Work is an incredibly simple time tracker for your terminal. Work helps you keep track of how much 
time you spend on specific tasks at work. It can be thought of as a user friendly wrapper around a
simple log file which contains either _start_ or _stop_ events.

## Installation
Work is still being developed, however it is highly usable in its current state since all core 
features have been implemented. As of now you are on your own when installing Work.

Instructions:
1. Clone the repository to your system using `git clone`.
2. Navigate to the root of the repository and run `cargo install --path .`.
3. If everything went well congratulations! Work is now installed on your system.

## Usage
Work keeps an event log where each line begins with a timestamp, the event type, project name, and 
a description. The Work executable helps you interact with this log.

The two main features of Work is to be intuitive and simple to use, this is why each command is a 
single english word conveying the intent of the command.

### Help
```
Work - Terminal Time Tracker! 0.1.0

USAGE:
    work <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    free       Exits with an error code of 0 if no work is in progress, and 1 otherwise
    help       Prints this message or the help of the given subcommand(s)
    of         Outputs a summary of work done within a given interval
    since      Appends a new event to the log that started at a given time
    start      Appends a new start event to the log
    status     Prints the status of the last event in the log in human readable form
    stop       Appends a new stop event to the log
    until      Appends an event to the log that stops at a given time
    while      Appends a start event, executes a given command, and then appends stop event once the command
               finishes
    working    Exits with an error code of 0 if work is in progress, and 1 otherwise
```

### Checking for status
You can check the current status of the log with the following commands:
* `status` for when you want to know what you are working on.
* `free` for when you want to check whether you are free or not.
* `working` for when you want to check whether you are working or not.

### Adding an event
Work interacts with the log by appending events to it. There is only one rule regarding the log: 
You can not enter the same type of event twice in a row. This means that if the last event in the
log is a _start_ event, you can only append a _stop_ event and vice versa.

You can append an event to the log with the following commands:
* `start` for starting a new project now.
* `stop` for stopping the current project.
* `since` for when you forgot to start a project some time ago.
* `until` for when you have decided to work for the next 3 hours (as an example).
* `while` for when you are starting a command that you want to track the time of (vim for example). 

### Reviewing past work
Most importantly Work allows you to review time spent on different projects with the `of` command.
For example you might want to know what you spent your time on today, then you simply execute: 
`work of today` and Work will show you how much time was spent on which projects.

## Acknowledgements
Work is inspired by [NineToFive](https://github.com/SuprDewd/NineToFive/), a lightweight command-line
application for keeping track of work hours.

Most of Work's features are directly from NineToFive!
