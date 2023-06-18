// #![allow(unused, unused_variables)]

use chrono::format::{strftime::StrftimeItems, DelayedFormat, ParseError};
use chrono::prelude::*;
use chrono::Duration;
use core::panic;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::time::SystemTime;

#[derive(Debug)]
pub struct Task {
    thing: String,
    status: TaskStatus,
    deadline: Deadline,
    label: String,
}

impl Task {
    fn add_task(tasks: &mut Vec<Task>) {
        println!("Type \"exit\" to break to the CLI user interface");

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

                parsed_deadline = match Deadline::new(&deadline.trim().to_string()) {
                    Ok(deadline) => deadline,
                    Err(err) => {
                        eprintln!("{}", err);
                        continue;
                    }
                };
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
                DateTime::date_dmy(task.deadline.date)
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

        // 1, 2, 3 4,5
        //

        let mut i = 0;
        while i < ids.len() - 1 {
            let first = ids[i];
            let second = ids[i + 1];
            let diff = second - first;
            println!("{diff}");
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

        println!("ids: {ids:?}\n vai: {available_ids:?}");

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

#[derive(Debug)]
struct Deadline {
    // isPostponed: bool,
    date: DateTime<Local>,
}

#[derive(Debug)]
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
        // format: DD/MM/YYYY HH:MM:SS
        // let current_time = Self::date_now();

        let splited = input
            .split_whitespace()
            .enumerate()
            .map(|(idx, x)| match idx {
                0 => match x.to_lowercase().as_str() {
                    "today" => {
                        let day_month_year = DateTime::date_dmy(DateTime::date_now());

                        Ok(day_month_year
                            .to_string()
                            .split("/")
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>())
                    }
                    "tomorrow" => {
                        let day_month_year =
                            DateTime::date_dmy(DateTime::date_now() + Duration::days(1));

                        Ok(day_month_year
                            .to_string()
                            .split("/")
                            .map(|x| x.to_string())
                            .collect::<_>())
                    }
                    _ => Ok(x.split("/").map(|x| x.to_string()).collect::<Vec<String>>()),
                },
                1 => Ok(x.split(":").map(|x| x.to_string()).collect::<Vec<String>>()),
                _ => Err("Invalid input; Example correct input: 10/06/2023 15:30"),
            })
            .collect::<Result<Vec<Vec<String>>, _>>();

        if let Ok(splited) = splited.as_ref() {
            for vec_num in splited.iter() {
                for num_as_str in vec_num.iter() {
                    if !num_as_str.chars().all(|num| num.is_numeric()) {
                        return Err("Invalid character detected: contains non-numeric character"
                            .to_string());
                    }

                    if let Err(_) = num_as_str.parse::<i32>() {
                        return Err("Failed to parse the number".to_string());
                    }
                }
            }
        } else {
            return Err("Missing value".to_string());
        }

        let date = splited
            .unwrap()
            .iter()
            .enumerate()
            .map(|(idx, x)| match idx {
                0 => Ok(x.join("/")).to_owned(),
                1 => Ok(x.join(":")).to_owned(),
                _ => Err("Wrong input"),
            })
            .collect::<Result<Vec<String>, _>>()?;
        match DateTime::parse_string_to_datetime_local(&date.join(" ").trim()) {
            Ok(date) => Ok(Self { date }),
            Err(err) => Err(format!("Error occurred while parsing the date. Make sure it's in the right format\nMessage: {}", err)),
        }
    }
}

// Or the switches in the near future and maybe
fn handle_action_by_argument(tasks: &Vec<Task>, switch: String) -> Result<(), String> {
    // && switch.chars().all(|c| c.is_alphanumeric())
    if switch.starts_with("--") {
        let switch = switch.trim_start_matches("--").to_string();

        match switch.as_str() {
            "1" | "show-tasks" => Ok(cli_manager::show_tasks(tasks)),
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
        let switch = &args[1..].join("").to_string();
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
            "1" | "show tasks" => cli_manager::show_tasks(tasks),
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

    // fn edit_deadline(task: &mut Task) {
    //     let new_value = prompt_field_for_task("deadline");
    // }
}

// fn show_completed() {
//     ()
// }

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
        // Possible inputs:

        // 10/06/2023 12:30
        // 10/06/2023 -> 10/06/2023 10:30
        // tomorrow 12:30 -> 06/06/2023 12:30
        // tomorrow
        // today 12:30
        // 12:30

        let mut composed_date: String = String::new();

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

    pub fn show_tasks(tasks: &Vec<Task>) {
        if tasks.len() == 0 {
            println!("No available tasks");
            return ();
        }

        for task in tasks {
            println!("{task}");
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

                let mut instance_fields_hashmap: HashMap<&str, &str> =
                    HashMap::from([("thing", thing_value)]);

                parsed.iter().for_each(|x| {
                    let field = x.get(0).unwrap();
                    let value = x.get(1).unwrap();

                    if *field == "deadline" {
                        let mut date = x.last().unwrap().split_whitespace();
                        let date = date.next().unwrap();

                        instance_fields_hashmap.insert(field, date.trim());
                    } else {
                        instance_fields_hashmap.insert(field, value);
                    }
                });

                parse_task_from_file(&mut instance_fields_hashmap)
            })
            .collect::<Vec<Task>>();

        Ok(instaces)
    }

    pub fn parse_task_from_file(instance_fields: &mut HashMap<&str, &str>) -> Task {
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
                date: match DateTime::parse_from_rfc3339(
                    instance_fields.remove("deadline").unwrap(),
                ) {
                    Ok(date) => date.with_timezone(&Local),
                    Err(err) => self::panic!("{err}"),
                },
            },
            label: instance_fields.remove("label").unwrap().to_string(),
        }
    }
}
