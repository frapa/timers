use clap;

mod util;
mod basic_op;
use basic_op::*;
mod list_op;
use list_op::*;
mod report_op;
use report_op::*;
mod import_export_op;
use import_export_op::*;

fn main() {
    let matches = parse_args();

    match matches.subcommand_name() {
        Some("log") => log_command(matches.subcommand_matches("log").unwrap()),
        Some("status") => status_command(matches.subcommand_matches("status").unwrap()),
        Some("stop") => stop_command(matches.subcommand_matches("stop").unwrap()),
        Some("report") => {
            let submatches = matches.subcommand_matches("report").unwrap();
            match submatches.subcommand_name() {
                Some("days") => report_days_command(submatches),
                _ => report_days_command(submatches),
            }
        },
        Some("tasks") => tasks_command(matches.subcommand_matches("tasks").unwrap()),
        Some("export") => export_command(matches.subcommand_matches("export").unwrap()),
        _ => {},
    }
}

fn parse_args() -> clap::ArgMatches<'static> {
    return clap::App::new("timers")
        .author("Francesco Pasa <francescopasa@gmail.com>")
        .version(clap::crate_version!())
        .about("Track time spent on tasks")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("log")
            .alias("start")
            .about("Log time on a task")
            .arg(clap::Arg::with_name("TASK")
                .required(true)
                .index(1)
                .help(
                    "Name of the task to log, or ID of an existing task, \
                        to continue logging on an existing task."
                )
            )
            .arg(clap::Arg::with_name("AT")
                .long("at")
                .takes_value(true)
                .value_name("TIME")
                .allow_hyphen_values(true)
                .help("Start logging at the specified time.")
            )
        )
        .subcommand(clap::SubCommand::with_name("status")
            .about("Get logging status")
            .arg(clap::Arg::with_name("watch")
                .short("w")
                .long("watch")
                .takes_value(true)
                .min_values(0)
                .max_values(1)
                .default_value("5")
                .help("Keep watching the status, for a GUI like effect.")
            )
            .arg(clap::Arg::with_name("timeline")
                .short("T")
                .long("timeline")
                .help("Print timeline with the current status today.")
            )
            .arg(clap::Arg::with_name("total")
                .long("tot")
                .takes_value(true)
                .help("Set the timeline span in hours. Defaults to 8.")
            )
        )
        .subcommand(clap::SubCommand::with_name("stop")
            .about("Stop logging time on the current task")
            .arg(clap::Arg::with_name("AT")
                .long("at")
                .takes_value(true)
                .value_name("TIME")
                .allow_hyphen_values(true)
                .help("Stop logging at the specified time.")
            )
        )
        .subcommand(clap::SubCommand::with_name("report")
            .about("Report statistics on the tasks")
            .subcommand(clap::SubCommand::with_name("days")
                .about("Report statistics on days.")
            )
            .arg(clap::Arg::with_name("no-tot")
                .long("--no-tot")
                .help("Omit printing totals")
            )
        )
        .subcommand(clap::SubCommand::with_name("tasks")
            .about("Print tasks")
            .arg(clap::Arg::with_name("long")
                .short("-l")
                .long("--long")
                .help("Display more information for each task.")
            )
            .arg(clap::Arg::with_name("num")
                .short("-n")
                .long("--num")
                .default_value("30")
                .help("Display the last <num> tasks. Default 30.")
            )
        )
        .subcommand(clap::SubCommand::with_name("export")
            .about("Export tasks to CSV")
            .arg(clap::Arg::with_name("OBJECT")
                .required(true)
                .index(1)
                .possible_values(&["logs", "tasks"])
                .help(
                    "Either 'logs', to export log information or 'tasks' \
                        to export task information."
                )
            )
            .arg(clap::Arg::with_name("output")
                .short("-o")
                .long("--output")
                .takes_value(true)
                .number_of_values(1)
                .help("Export to the given file instead of printing to standard output.")
            )
            .arg(clap::Arg::with_name("delimiter")
                .short("-d")
                .long("--delimiter")
                .takes_value(true)
                .number_of_values(1)
                .default_value(",")
                .help("Field delimiter to use. Only a single character is allowed.")
            )
            .arg(clap::Arg::with_name("from")
                .long("--from")
                .takes_value(true)
                .number_of_values(1)
                .help("Export only starting from the given date and time.")
            )
            .arg(clap::Arg::with_name("to")
                .long("--to")
                .takes_value(true)
                .number_of_values(1)
                .help("Export only up to the given date and time.")
            )
        )
        .get_matches();
}