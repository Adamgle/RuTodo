// #![allow(unused, unused_variables)]

use chrono::format::{strftime::StrftimeItems, DelayedFormat, ParseError};
use chrono::prelude::*;
use chrono::Duration;
use core::panic;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::SystemTime;
use std::{fmt, vec};

mod utils;

#[derive(Debug, Clone)]
pub struct Task {
    thing: String,
    status: TaskStatus,
    deadline: Deadline,
    label: String,
}

impl Task {
    fn add_task(tasks: &mut Vec<Task>) {
        cli_manager::clear_console();
        println!("Type \"exit\" to break to the CLI user interface");
        println!("{}{}", "Thing: String\n", 
        "Deadline: format: 10/06/2023 12:30 | 10/06/2023 | tomorrow 12:30 | today 12:30 | tomorrow | 06/06/2023 | 12:30");

        'outer: loop {
            let parsed_deadline;
            let thing = cli_manager::get_labeled_input_from_user("Thing");

            if thing.is_empty() {
                eprintln!("Task thing cannot be empty");
                continue;
            }

            if thing.trim() == "exit" {
                cli_manager::clear_console_and_display_help();
                break;
            }

            loop {
                let deadline = cli_manager::get_labeled_input_from_user("Deadline");

                if deadline.trim() == "exit" {
                    cli_manager::clear_console_and_display_help();
                    break 'outer;
                }

                parsed_deadline = match Deadline::new(&deadline) {
                    Ok(deadline) => deadline,
                    Err(err) => {
                        eprintln!("{err}");
                        continue;
                    }
                };
                // parsed_deadline = Deadline::new(&deadline);
                break;
            }

            let mut all_ids = Task::get_all_ids(&tasks);

            let available_ids = Task::find_available_ids(&mut all_ids);

            let label = if !available_ids.is_empty() {
                format!("Task {}", available_ids.first().unwrap())
            } else {
                format!("Task {}", tasks.len() + 1)
            };

            let task = Task {
                thing: format!("\"{}\"", thing),
                status: TaskStatus::Todo,
                deadline: parsed_deadline,
                label,
            };

            println!(
                "Task successfully added:\nTask {{ thing: {}, status: {:?}, deadline: {} }}",
                task.thing,
                task.status,
                DateTime::date_user_formating(task.deadline.date)
            );

            tasks.push(task);

            if let Err(err) = tasks_file_manager::save_tasks(&tasks) {
                eprint!("{err}");
            };
        }
    }

    fn edit_task(
        tasks: &mut Vec<Task>,
        task_label_number: String,
        switch_field: &Option<String>,
    ) -> Result<(), String> {
        let task_labeled_by = format!("Task {task_label_number}");
        let mut is_switch_invalid = false;

        if let Some(task) = tasks.into_iter().find(|x| x.label == task_labeled_by) {
            match task.status {
                TaskStatus::Aborted(_) => {
                    return Err("Cannot edit task with previous status as Aborted".to_string());
                }
                TaskStatus::Expired(_) => {
                    return Err("Cannot edit task with previous status as Expired".to_string());
                }
                _ => loop {
                    let mut field_to_edit: String = String::new();

                    if !is_switch_invalid {
                        if let Some(switch) = switch_field {
                            let switch = switch.trim_start_matches("--").to_string();
                            match switch.as_str() {
                                "thing" | "status" => field_to_edit = switch,
                                _ => {
                                    eprintln!("No such field to edit, you lying son of a bitch!");
                                    is_switch_invalid = true;
                                }
                            }
                        };
                    }

                    if field_to_edit.is_empty() {
                        field_to_edit = cli_manager::get_labeled_input_from_user("field")
                            .to_lowercase()
                            .to_string()
                    }

                    if field_to_edit == "thing" {
                        EditTaskConfig::edit_thing(task)
                    } else if field_to_edit == "status" {
                        EditTaskConfig::edit_status(task)
                    } else if field_to_edit == "exit" {
                        cli_manager::clear_console_and_display_help();
                        break;
                    } else {
                        eprintln!("No such field to edit, you lying son of a bitch!");
                        continue;
                        // break;
                    }

                    if let Err(err) = tasks_file_manager::save_tasks(&tasks) {
                        eprintln!("{err}");
                    };

                    break;
                },
            }
        } else {
            return Err("Could not found Task with this label".to_string());
        }
        Ok(())
    }

    fn delete_task(tasks: &mut Vec<Task>, task_label_number: String) -> Result<(), String> {
        match task_label_number.as_str() {
            "all" => {
                tasks.clear();
                println!("Successfully deleted all tasks");
            }
            task_number => {
                let task_labeled_by = format!("Task {task_number}");
                if tasks.iter().any(|x| x.label == task_labeled_by) {
                    tasks.retain(|x| *x.label != task_labeled_by);
                } else {
                    eprintln!("Task with label {task_labeled_by} does not exists");
                }
            }
        };

        if let Err(err) = tasks_file_manager::save_tasks(&tasks) {
            eprintln!("{err}");
        }

        Ok(())
    }

    fn find_available_ids(ids: &mut Vec<i32>) -> Vec<i32> {
        if ids.len() == 0 {
            return vec![];
        }
        let mut available_ids = Vec::new();

        ids.sort();

        let mut i = 0;
        while i < ids.len() - 1 {
            let first = ids[i];
            let second = ids[i + 1];
            let diff = second - first;
            if diff > 1 {
                let slice_of_ids: Vec<i32> = (first + 1..second).collect();
                available_ids.extend_from_slice(&slice_of_ids);
                i += (diff - 1) as usize;
            }

            i += 1;
        }

        // pad all ids from the 1 to smallest present id
        if ids.len() >= 1 {
            for id in 1..ids.first().unwrap().to_owned() {
                available_ids.push(id);
            }
        }

        available_ids
    }

    fn get_all_ids(tasks: &Vec<Task>) -> Vec<i32> {
        tasks
            .iter()
            .filter_map(|task| {
                task.label
                    .chars()
                    .last()?
                    .to_digit(10)
                    .map(|digit| digit as i32)
            })
            .collect()
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\nThing: {}\nStatus: {}\nDeadline: {}\n",
            self.label,
            self.thing
                .strip_prefix("\"")
                .and_then(|s| s.strip_suffix("\""))
                .unwrap(),
            self.status,
            DateTime::date_user_formating(self.deadline.date),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct Deadline {
    // isPostponed: bool,
    date: DateTime<Local>,
}

#[derive(Debug, Clone, Copy)]
enum TaskStatus {
    Completed,
    Todo,
    Postponed(DateTime<Local>),
    Expired(DateTime<Local>),
    Aborted(DateTime<Local>),
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TaskStatus::Postponed(date) => {
                write!(f, "Postponed({})", DateTime::date_user_formating(date))
            }
            TaskStatus::Expired(date) => {
                write!(f, "Expired({})", DateTime::date_user_formating(date))
            }
            TaskStatus::Aborted(date) => {
                write!(f, "Aborted({})", DateTime::date_user_formating(date))
            }
            ref other => write!(f, "{:?}", other),
        }
    }
}

impl Deadline {
    fn new(input: &String) -> Result<Self, String> {
        match DateTime::parse_formated_string_to_datetime(input.to_owned(), DateTime::date_now()) {
            Ok(date) => Ok(Self { date }),
            Err(err) => Err(format!("Error occurred while parsing the date. Make sure it's in the right format:\n{}\nMessage: {}", 
            "Deadline: format: 10/06/2023 12:30 | 10/06/2023 | tomorrow 12:30 | today 12:30 | tomorrow | 06/06/2023 | 12:30", err)),
        }
    }
}

// Or the switches in the near future and maybe
fn handle_action_by_argument(tasks: &mut Vec<Task>, switch: String) -> Result<(), String> {
    if switch.starts_with("--") {
        // TEMPORARY SOLUTION
        // WILL HANDLE IT AS SEPERATE FUNCTION TO FIND WHERE SWITCH STARTS AND ENDS WITH ALL OF IT'S ARGUMENTS
        let switch_seperated = switch
            .trim_start_matches("--")
            .split_whitespace()
            .next()
            .unwrap()
            .to_string();
        let arg_to_switch = switch.split(" ").last().unwrap();

        match switch_seperated.as_str() {
            "1" | "show-tasks" => Ok(cli_manager::show_tasks(tasks, None)?),
            "parse" => {
                utils::parse_redirected_stream_of_show_tasks(tasks, PathBuf::from(arg_to_switch))
                    .map_err(|err| err.to_string())
            }
            _ => Err("swtich does not exists".to_string()),
        }
    } else {
        return Err("Switch has a wrong format or does not exists".to_string());
    }?;

    Ok(())
}

pub fn spawn_cli_interface(tasks: &mut Vec<Task>) -> Result<(), String> {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        let switch = &args[1..].join(" ").to_string();
        return handle_action_by_argument(tasks, switch.to_owned());
    }

    cli_manager::show_user_actions();

    loop {
        let mut action = String::new();

        print!("prompt: ");
        io::stdout().flush().expect("Failed to flush stdout");

        io::stdin()
            .read_line(&mut action)
            .expect("Unable to write to the writtable buffer");

        match action.trim().to_lowercase().to_string().as_str() {
            action
                if action == "1"
                    || action == "show tasks"
                    || action.starts_with("show tasks ")
                    || action.starts_with("1 ")
                    || action.starts_with("show task ") =>
            {
                if action == "show tasks" || action == "1" {
                    if let Err(err) = cli_manager::show_tasks(tasks, None) {
                        eprintln!("Error: {err}");
                    }
                    continue;
                }

                // println!("Working... {:?}", action.split("").collect::<Vec<_>>());

                match action {
                    action if action.starts_with("show tasks ") || action.starts_with("1 ") => {
                        let mut switches: Vec<(String, Option<&[&str]>)> = Vec::new();

                        let switches_with_values = action
                            .trim_start_matches("show tasks ")
                            .trim_start_matches("1 ");

                        let switches_with_values =
                            switches_with_values.split_whitespace().collect::<Vec<_>>();

                        let mut indexes_to_slice_with = switches_with_values
                            .iter()
                            .enumerate()
                            .map(|(idx, x)| if x.starts_with("--") { idx as i32 } else { -1 })
                            .filter(|&x| x >= 0)
                            .map(|x| x as usize)
                            .collect::<Vec<_>>();

                        // explicitly pad rest index (equivalent to switches_with_values vector length)
                        indexes_to_slice_with.push(switches_with_values.len());

                        let mut ranges: Vec<(usize, usize)> = vec![];
                        let mut idx = 0;

                        // indexes_to_slice_with.len() - 1 'cause we are extending this vector explicitly
                        while ranges.len() != indexes_to_slice_with.len() - 1 {
                            let slice_from = indexes_to_slice_with[idx];
                            let slice_to = match indexes_to_slice_with.get(idx + 1) {
                                Some(slice_to_idx) => slice_to_idx.to_owned(),
                                None => slice_from,
                            };

                            idx += 1;

                            let range = (slice_from, slice_to);
                            ranges.push(range);
                        }

                        for (from, to) in ranges {
                            let switch_args_pair = &switches_with_values[from..to];

                            let switch = switch_args_pair[0].to_string();

                            if switch_args_pair.len() > 1 {
                                let args = &switch_args_pair[1..];
                                switches.push((switch, Some(args)))
                            } else {
                                switches.push((switch, None))
                            }
                        }
                        if let Err(err) = cli_manager::show_tasks(tasks, Some(switches)) {
                            eprintln!("Error: {err}");
                        }
                    }
                    "show task " => (),
                    _ => (),
                };
            }
            "2" | "add task" | "add" => Task::add_task(tasks),
            action
                if action.starts_with("3 ")
                    || action.starts_with("edit task ")
                    || action.starts_with("edit ") =>
            {
                let user_input = action
                    .trim_start_matches("3 ")
                    .trim_start_matches("edit task ")
                    .trim_start_matches("edit ")
                    .trim()
                    .to_string();

                let splited = user_input.split(" ").collect::<Vec<&str>>();

                let (task_number, switch_field) = if splited.len() == 1 {
                    (user_input, None)
                } else if splited.len() == 2 && splited[1].starts_with("--") {
                    (splited[0].to_string(), Some(splited[1].to_string()))
                } else {
                    (String::new(), None)
                };

                if task_number.chars().all(|c| c.is_numeric()) {
                    if let Err(err) = Task::edit_task(tasks, task_number, &switch_field) {
                        eprintln!("{err}");
                    }
                } else {
                    eprintln!("Invalid task number")
                }
            }
            action
                if action.starts_with("4 ")
                    | action.starts_with("delete task ")
                    | action.starts_with("delete ") =>
            {
                // task_number =| numeric_string | "all"
                let mut task_number = action
                    .trim_start_matches("4 ")
                    .trim_start_matches("delete task ")
                    .trim_start_matches("delete ")
                    .trim()
                    .to_string();
                task_number =
                    match task_number.chars().all(|c| c.is_numeric()) || task_number == "all" {
                        true => task_number,
                        false => {
                            eprintln!("Invalid task number");
                            continue;
                        }
                    };

                if let Err(err) = Task::delete_task(tasks, task_number) {
                    eprintln!("{err}");
                }
            }
            "exit" => std::process::exit(0),
            "help" => cli_manager::show_user_actions(),
            "cls" => cli_manager::clear_console(),
            _ => eprintln!("Unrecognized program action"),
        };
    }
}

struct EditTaskConfig;

impl EditTaskConfig {
    fn edit_thing(task: &mut Task) {
        let new_value = cli_manager::get_labeled_input_from_user("thing");
        if new_value == "exit" {
            cli_manager::clear_console_and_display_help();
            return ();
        }

        task.thing = format!("\"{}\"", new_value);

        cli_manager::clear_console();

        println!("Updated task:\n{task}");
    }

    fn edit_status(task: &mut Task) {
        // print available enums help message
        let help_message = || {
            cli_manager::clear_console();

            println!(
                "Available options: {}",
                r#"
TaskStatus {
    Completed,
    Todo,
    Postponed(Date(relative to the previous date) -> format: 10/06/2023 12:30 | 10/06/2023 | tomorrow 12:30 | today 12:30 | tomorrow | 06/06/2023 | 12:30),
    Aborted
}
help => spawn this message"#
            );
        };
        help_message();

        use TaskStatus::{Aborted, Completed, Postponed, Todo};

        loop {
            let new_value = cli_manager::get_labeled_input_from_user("status")
                .to_lowercase()
                .to_string();

            if new_value == "help" {
                help_message();
                continue;
            } else if new_value == "exit" {
                cli_manager::clear_console_and_display_help();
                break;
            }

            let new_status = match new_value.as_str() {
                "completed" => Completed,
                "todo" => Todo,
                new_value if new_value.starts_with("postponed ") => {
                    let date_part = &new_value.split(" ").collect::<Vec<_>>()[1..];

                    let date_from = match task.status {
                        Postponed(date) => date,
                        _ => task.deadline.date,
                    };

                    let new_date =
                        DateTime::parse_formated_string_to_datetime(date_part.join(" "), date_from);

                    match new_date {
                        Ok(date) => Postponed(date),
                        Err(_) => {
                            continue;
                        }
                    }
                }
                // "expired" => match task.status {
                //     Postponed(date) => Expired(date.clone()),
                //     _ => Expired(task.deadline.date.clone()),
                // },
                "aborted" => Aborted(DateTime::date_now()),
                _ => {
                    eprintln!("No such status available");
                    continue;
                }
            };
            task.status = new_status;

            cli_manager::clear_console();

            println!("Updated task:\n{task}");

            break;
        }
    }
}

trait DateTimeFormatter {
    fn is_valid_date_format(date_string: &str) -> bool;
    fn is_valid_dmy_format(date_string: &str) -> bool;
    fn is_valid_hm_format(date_string: &str) -> bool;
    fn parse_string_to_datetime_local(date_string: &str) -> Result<DateTime<Local>, ParseError>;
    fn date_now() -> DateTime<Local>;
    fn date_dmy<'a>(date: DateTime<Local>) -> DelayedFormat<StrftimeItems<'a>>;
    fn date_user_formating(date: DateTime<Local>) -> String;
    fn parse_formated_string_to_datetime(
        date: String,
        date_from: DateTime<Local>,
    ) -> Result<DateTime<Local>, String>;
}

impl DateTimeFormatter for DateTime<Local> {
    fn is_valid_date_format(date_string: &str) -> bool {
        let format = "%d/%m/%Y %H:%M";

        match NaiveDateTime::parse_from_str(date_string, format) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    fn is_valid_dmy_format(date_string: &str) -> bool {
        if let Ok(dt) = NaiveDate::parse_from_str(date_string, "%d/%m/%Y") {
            // Check if the parsed date matches the original input
            let formatted = dt.format("%d/%m/%Y").to_string();
            formatted == date_string
        } else {
            false
        }
    }

    fn is_valid_hm_format(date_string: &str) -> bool {
        if let Ok(parsed_time) = NaiveTime::parse_from_str(date_string, "%H:%M") {
            if let Some(_) = NaiveTime::from_hms_opt(parsed_time.hour(), parsed_time.minute(), 0) {
                return true;
            }
        }
        false
    }

    fn parse_string_to_datetime_local(date_string: &str) -> Result<DateTime<Local>, ParseError> {
        let format = "%d/%m/%Y %H:%M";

        let naive_datetime = NaiveDateTime::parse_from_str(date_string, format)?;

        let datetime_local = Local
            .from_local_datetime(&naive_datetime)
            .single()
            .expect("Ambiguous or non-existent local time");

        Ok(datetime_local)
    }

    // returns d/m/y format
    fn date_now() -> DateTime<Local> {
        let curr_time = SystemTime::now();
        let dt: DateTime<Local> = curr_time.clone().into();
        dt
    }

    fn date_dmy<'a>(date: DateTime<Local>) -> DelayedFormat<StrftimeItems<'a>> {
        return date.format("%d/%m/%Y");
    }

    fn date_user_formating(date: DateTime<Local>) -> String {
        return date.format("%d/%m/%Y %H:%M").to_string();
    }

    fn parse_formated_string_to_datetime(
        date: String,
        date_from: DateTime<Local>,
    ) -> Result<DateTime<Local>, String> {
        let mut composed_date: String = String::new();

        let date = date.to_lowercase().to_string();
        let date_parts = date.split(" ").collect::<Vec<&str>>();

        if date == "tomorrow" || date == "today" || date_parts.len() == 2 {
            let mut hm = date_from.format("%H:%M").to_string();

            if date_parts.len() == 2 {
                hm = date_parts[1].to_string();
            }

            match date_parts[0] {
                "tomorrow" => {
                    let relative_tomorrow =
                        DateTime::date_dmy(date_from + Duration::days(1)).to_string();
                    composed_date = format!("{} {hm}", relative_tomorrow);
                }
                "today" => {
                    composed_date = format!("{} {hm}", DateTime::date_dmy(date_from).to_string())
                }
                _ => (),
            };
        } else if DateTime::is_valid_dmy_format(&date) {
            let hm = date_from.format("%H:%M").to_string();
            composed_date = format!("{date} {hm}");
        } else if DateTime::is_valid_hm_format(&date) {
            let dmy = DateTime::date_dmy(date_from).to_string();
            composed_date = format!("{dmy} {date}");
        }

        match DateTime::parse_string_to_datetime_local(if composed_date.is_empty() {
            &date
        } else {
            &composed_date
        }) {
            Ok(date) => Ok(date),
            Err(err) => Err(err.to_string()),
        }
    }
}

pub mod cli_manager {
    use super::*;

    pub fn get_labeled_input_from_user(field_name: &str) -> String {
        let mut input = String::new();

        if !field_name.is_empty() {
            print!("{field_name}: ");
        }

        io::stdout().flush().expect("Failed to flush stdout");

        std::io::stdin()
            .lock()
            .read_line(&mut input)
            .expect("Unable to write to the writtable buffer");

        return input.trim().to_string();
    }

    pub fn show_tasks(
        tasks: &Vec<Task>,
        switches: Option<Vec<(String, Option<&[&str]>)>>,
    ) -> Result<(), String> {
        // show tasks | 1 => show tasks
        // show task 1 | 1 1 => show task with specific id (label id)
        // show tasks --thing "adasda sdas d asdasd" => show tasks that includes this string in thing field
        // show tasks --status Postponed => show tasks with status as postpoend
        // show tasks --deadline => sort tasks by deadline date
        // show tasks --deadline tomorrow => show tasks with deadline that is tomorrow, ignoring hours:minutes
        // show tasks --deadline tomorrow 12:30 => show tasks with deadline as tomorrow 12:30
        // show tasks --status postponed => match every task with status as postponed
        // show tasks --status postponed --date => match every task with status as postponed and sort it by date
        // show tasks --alphabetical | --alph
        // show tasks --date > 10/06/2023

        if tasks.len() == 0 {
            println!("No available tasks");
            return Ok(());
        }

        match switches {
            Some(switches) => {
                for switch_args_pair_option in switches.iter() {
                    let (switch, args) = (
                        switch_args_pair_option.0.to_owned(),
                        switch_args_pair_option.1,
                    );
                    let switch = switch.to_lowercase();
                    let switch = switch.trim_start_matches("--");

                    match switch {
                        "thing" | "status" => {
                            if args.is_none() {
                                return Err(format!(
                                    "switch {switch} requiers additional arguments"
                                ));
                            }
                        }
                        _ => (),
                    };

                    // We have to clone tasks to operate on own version of vec to make the function reqursive and type valid
                    let mut tasks_clone = tasks.clone();

                    println!("Tasks: {}, Tasks_clone: {}", tasks.len(), tasks_clone.len());

                    let filtered_by_switch = match switch {
                        "thing" => {
                            let thing = args.unwrap().join("");

                            let filtered_by_switch = tasks_clone
                                .into_iter()
                                .filter(|task| {
                                    task.thing
                                        .strip_prefix("\"")
                                        .and_then(|s| s.strip_suffix("\""))
                                        .unwrap()
                                        .to_string()
                                        .to_lowercase()
                                        .starts_with(&thing.to_lowercase())
                                })
                                .collect::<Vec<_>>();

                            filtered_by_switch
                        }
                        "status" => {
                            let status = args.unwrap().join("");

                            let filtered_by_switch = tasks_clone
                                .into_iter()
                                .filter(|task| match_status(task, &status))
                                .collect::<Vec<_>>();

                            filtered_by_switch
                        }
                        "deadline" => {
                            match args {
                                Some(_) => {
                                    return Err(
                                        "This switch does not take any additional arguments"
                                            .to_string(),
                                    )
                                }
                                None => tasks_clone.sort_by_key(|task| task.deadline.date),
                            }
                            tasks_clone
                        }
                        "date" => match args {
                            Some(args) => {
                                let parsable_date_format = args.join(" ").to_string();
                                let input_date = DateTime::parse_formated_string_to_datetime(
                                    parsable_date_format,
                                    DateTime::date_now(),
                                )?;

                                let filtered_by_switch = tasks_clone
                                    .into_iter()
                                    .filter(|task| match task.status {
                                        TaskStatus::Postponed(date)
                                        | TaskStatus::Expired(date)
                                        | TaskStatus::Aborted(date) => {
                                            date.day() == input_date.day()
                                                && date.month() == input_date.month()
                                                && date.year() == input_date.year()
                                        }
                                        _ => {
                                            task.deadline.date.day() == input_date.day()
                                                && task.deadline.date.month() == input_date.month()
                                                && task.deadline.date.year() == input_date.year()
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                filtered_by_switch
                            }
                            None => {
                                tasks_clone.sort_by_key(|task| match task.status {
                                    TaskStatus::Postponed(date)
                                    | TaskStatus::Expired(date)
                                    | TaskStatus::Aborted(date) => date,
                                    _ => task.deadline.date,
                                });
                                tasks_clone
                            }
                        },

                        _ => return Err("inexsistent switch".to_string()),
                    };

                    // We are slicing switch that just executed
                    let advanced_status_vector = switches[1..].to_vec();

                    // If there are no switches left, we're returning None
                    // and returning output
                    match advanced_status_vector.len() {
                        0 => show_tasks(&filtered_by_switch, None)?,
                        _ => show_tasks(&filtered_by_switch, Some(advanced_status_vector))?,
                    };
                }
            }
            None => {
                println!("{}", tasks.len());
                // for task in tasks {
                //     println!("{task}");
                // }
                return Ok(());
            }
        };
        Ok(())
    }

    fn match_status(task: &Task, status: &str) -> bool {
        match status {
            "completed" => matches!(task.status, TaskStatus::Completed),
            "todo" => matches!(task.status, TaskStatus::Todo),
            "postponed" => matches!(task.status, TaskStatus::Postponed(_)),
            "expired" => matches!(task.status, TaskStatus::Expired(_)),
            "aborted" => matches!(task.status, TaskStatus::Aborted(_)),
            _ => false,
        }
    }

    pub fn show_user_actions() {
        println!(
            "Available actions:\n{}{}{}{}{}{}",
            "1 | show tasks => Display all tasks\n",
            "2 | add task | add => Add new task (thing, deadline) \n",
            "3 | edit task | edit => Edit task <Task id> [--field] \n",
            "4 | delete task | delete | delete all => Detete Task <Task id | all>\n",
            "help => Display this help message\n",
            "exit => Terminates current procces\n",
        )
    }

    pub fn clear_console() {
        if cfg!(target_os = "windows") {
            let _ = std::process::Command::new("cmd")
                .args(&["/C", "cls"])
                .status();
        } else {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg("clear")
                .status();
        }
    }

    pub fn clear_console_and_display_help() {
        cli_manager::clear_console();
        cli_manager::show_user_actions();
    }
}

pub mod tasks_file_manager {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    pub fn get_tasks_file_path() -> Result<PathBuf, Box<dyn Error>> {
        let dir_path = match std::env::var("SystemDrive") {
            Ok(system_drive_letter) => {
                let username = whoami::username();
                let path = format!(
                    "{}/Users/{}/documents/rust-todo/",
                    system_drive_letter, username
                );
                PathBuf::from(path)
            }
            Err(err) => {
                eprintln!("Error while reading SystemDrive env var with error: {err}");

                let working_dir = env::current_dir()?;

                println!(
                    "Using current directory as the storage for tasks: {}",
                    working_dir.display()
                );

                working_dir
            }
        };

        if dir_path.try_exists().is_ok() {
            fs::create_dir_all(&dir_path)?;
        }

        let file_path = PathBuf::from(format!("{}tasks.txt", dir_path.display()));
        // println!("Using as file: {}", file_path.display());

        Ok(file_path)
    }

    pub fn save_tasks(tasks: &Vec<Task>) -> Result<(), Box<dyn Error>> {
        let file_path = get_tasks_file_path()?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file_path)?;

        if tasks.is_empty() {
            file.set_len(0)?;
            return Ok(());
        }

        for task in tasks {
            file.write_fmt(format_args!(
                "Task {{ thing: {}, status: {:?}, label: {}, deadline: {:?} }}\n",
                task.thing, task.status, task.label, task.deadline
            ))?;
            file.flush().expect("Failed to flush buffer");
        }

        Ok(())
    }

    pub fn get_saved_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
        let file_path = get_tasks_file_path()?;

        let file = OpenOptions::new().read(true).open(file_path)?;

        let reader = BufReader::new(file);

        let instaces: Vec<Task> = reader
            .lines()
            .map(|line| {
                let line = line.unwrap();
                let parsed = line.split("Task { ");
                let parsed = parsed.filter(|x| x.len() > 0);
                let parsed = parsed.collect::<String>();

                let mut end = 0;

                // Handle parsing "thing" field separately
                let delimited = parsed.split_inclusive(",").collect::<Vec<_>>();

                for (idx, item) in delimited.iter().enumerate() {
                    if item.trim().to_lowercase().contains("status") {
                        end = idx;
                        break;
                    }
                }

                let thing_key_value_str = &delimited[0..end].join("");
                let thing_value = thing_key_value_str
                    .split("thing: ")
                    .collect::<Vec<_>>()
                    .join("");
                let thing_value = thing_value.trim_end_matches(",");

                let thing_key_value = format!("thing: {},", thing_value);
                let thing_key_value = thing_key_value.as_str();

                let parsed = parsed.replace(thing_key_value, "");
                let parsed = parsed
                    .split(",")
                    .map(|x| x.trim().split(": ").collect::<Vec<_>>())
                    .collect::<Vec<Vec<&str>>>();

                let mut instance_fields_hashmap: HashMap<String, String> =
                    HashMap::from([("thing".to_string(), thing_value.to_string())]);

                parsed.iter().for_each(|x| {
                    let field = x.get(0).unwrap().to_string();
                    let value = x.get(1).unwrap().to_string();

                    if field == "deadline" {
                        let mut date = x.last().unwrap().split_whitespace();
                        let date = date.next().unwrap();

                        instance_fields_hashmap.insert(field, date.trim().to_string());
                    } else {
                        instance_fields_hashmap.insert(field, value);
                    }
                });

                parse_task_from_file(&mut instance_fields_hashmap)
            })
            .collect::<Vec<Task>>();

        Ok(instaces)
    }

    pub fn parse_task_from_file(instance_fields: &mut HashMap<String, String>) -> Task {
        let deadline_date = instance_fields.remove("deadline").unwrap();
        Task {
            thing: instance_fields.remove("thing").unwrap().to_string(),
            status: match instance_fields
                .remove("status")
                .unwrap()
                .to_lowercase()
                .to_string()
                .as_str()
            {
                "completed" => TaskStatus::Completed,
                "todo" => TaskStatus::Todo,
                other_status => {
                    let parted = other_status.split("(").collect::<Vec<_>>();

                    if let [status, date] = parted.as_slice() {
                        let date = date.trim_end_matches(")");

                        match status.to_lowercase().to_string().as_str() {
                            "postponed" => TaskStatus::Postponed(
                                DateTime::parse_from_rfc3339(date)
                                    .unwrap()
                                    .with_timezone(&Local),
                            ),
                            "expired" => TaskStatus::Expired(
                                DateTime::parse_from_rfc3339(date)
                                    .unwrap()
                                    .with_timezone(&Local),
                            ),
                            "aborted" => TaskStatus::Aborted(
                                DateTime::parse_from_rfc3339(date)
                                    .unwrap()
                                    .with_timezone(&Local),
                            ),
                            _ => TaskStatus::Todo,
                        }
                    } else {
                        TaskStatus::Todo
                    }
                }
            },
            deadline: Deadline {
                date: if DateTime::is_valid_date_format(&deadline_date) {
                    match DateTime::parse_string_to_datetime_local(&deadline_date) {
                        Ok(date) => date,
                        Err(err) => self::panic!("{err}"),
                    }
                } else {
                    match DateTime::parse_from_rfc3339(&deadline_date) {
                        Ok(date) => date.with_timezone(&Local),
                        Err(err) => self::panic!("{err}"),
                    }
                },
            },
            label: instance_fields.remove("label").unwrap().to_string(),
        }
    }
}
