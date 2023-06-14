#![allow(unused, unused_variables)]

use crate::cli_manager::clear_console;
use chrono::format::{strftime::StrftimeItems, DelayedFormat, ParseError};
use chrono::prelude::*;
use chrono::Duration;
use core::panic;
use nanoid::nanoid;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PrefixComponent;
use std::time::SystemTime;

pub struct Tasks {
    tasks: Vec<Task>,
}

#[derive(Debug)]
pub struct Task {
    thing: String,
    status: TaskStatus,
    deadline: Deadline,
    label: String,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\nThing: {}\nStatus: {}\nDeadline: {}\n",
            self.label,
            self.thing,
            self.status,
            // match self.status {
            //     TaskStatus::Postponed(date) => DateTime::date_user_formating(date),
            //     TaskStatus::Expired(date) => DateTime::date_user_formating(date),
            //     TaskStatus::Aborted(date) => DateTime::date_user_formating(date),
            //     _ => format!("{}", self.status),
            // },
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

pub fn spawn_cli_interface(tasks: &mut Vec<Task>) -> Result<(), String> {
    cli_manager::show_user_actions();

    loop {
        let mut action = String::new();

        print!("prompt: ");
        io::stdout().flush().expect("Failed to flush stdout");

        io::stdin()
            .read_line(&mut action)
            .expect("Unable to write to the writtable buffer");

        // clear_console();
        // [1,2,3,4,5]
        // Task 1, Task 2
        let precompiled = "Task 1";

        match action.trim().to_lowercase().to_string().as_str() {
            "1" | "show tasks" => cli_manager::show_tasks(tasks),
            "2" | "add task" | "add" => add_new_task(tasks),
            // this to cli user help
            // "3" | "edit task" | "edit" => edit_task(tasks, action),
            action
                if action.starts_with("3 ")
                    || action.starts_with("edit task ")
                    || action.starts_with("edit ") =>
            {
                let task_number = action
                    .trim_start_matches("3 ")
                    .trim_start_matches("edit task ")
                    .trim_start_matches("edit ")
                    .trim()
                    .to_string();

                if task_number.chars().all(|c| c.is_numeric()) {
                    edit_task(tasks, task_number)
                }
            }
            // precompiled => edit_task(tasks, action),
            "exit" => std::process::exit(0),
            "help" => cli_manager::show_user_actions(),
            _ => eprintln!("Unrecognized program action"),
        };
    }
}

fn clear_console_and_display_help() {
    cli_manager::clear_console();
    cli_manager::show_user_actions();
}

fn prompt_field_for_task(field_name: &str) -> String {
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

fn add_new_task(tasks: &mut Vec<Task>) {
    println!("Type \"exit\" to break to the CLI user interface");
    // let (thing, deadline) = (&mut (), &mut ());

    'outer: loop {
        let mut parsed_deadline;
        let thing = prompt_field_for_task("Thing");

        if thing.trim() == "exit" {
            clear_console_and_display_help();
            break;
        }

        loop {
            let deadline = prompt_field_for_task("Deadline");

            if deadline.trim() == "exit" {
                clear_console_and_display_help();
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

        let mut val = get_all_ids(&tasks);

        let av_ids = find_available_ids(&mut val);

        let label = if !av_ids.is_empty() {
            format!("Task {}", av_ids.first().unwrap())
        } else {
            format!("Task {}", tasks.len() + 1)
        };

        let mut task = Task {
            thing,
            status: TaskStatus::Todo,
            deadline: parsed_deadline,
            // naive approach
            label,
        };

        println!(
            "Task successfully added:\nTask {{ thing: {}, status: {:?}, deadline: {} }}",
            task.thing,
            task.status,
            DateTime::date_dmy(task.deadline.date)
        );

        tasks.push(task);

        tasks_file_manager::save_tasks(&tasks);
    }
}

struct EditTaskConfig;

impl EditTaskConfig {
    fn edit_thing(task: &mut Task) {
        let new_value = prompt_field_for_task("thing");
        task.thing = new_value;
    }

    fn edit_status(task: &mut Task) {
        clear_console();
        // print available enums help message
        println!(
            "Available options: {}",
            r#"
TaskStatus {
    Completed,
    Todo,
    Postponed(Date -> format: d/m/Y H:M),
    Expired(Date -> format: d/m/Y H:M),
}"#
        );

        loop {
            let new_value = prompt_field_for_task("status").to_lowercase().to_string();

            use TaskStatus::{Aborted, Completed, Expired, Postponed, Todo};

            let new_status = match new_value.as_str() {
                "completed" => Completed,
                "todo" => Todo,
                new_value if new_value.starts_with("postponed ") => {
                    let date_parts = &new_value.split(" ").collect::<Vec<_>>()[1..];
                    let mut new_date: Result<Deadline, String> = Err(String::from(
                        "Could not compose new date for this status for this input; make sure it's in correct format -> postponed 14/06/2023 15:30",
                    ));

                    // if second part cannot be parsed into dmy part than we'll try to parse it into the hm part
                    // if this does not succeed then we'll throw an error

                    if date_parts.len() == 2 {
                        let date = date_parts.join(",");
                        new_date = Deadline::new(&date);
                    } else if date_parts.len() == 1 {
                        let date_part = date_parts.join(",");

                        if date_part == "tomorrow" || date_part == "today" {
                            // If the date_parts is size of 2 and this statement matches
                            // then we could infere the value of rest of the date which is hours:minues from before date
                            let curr_deadline_hm = &task.deadline.date.format("%H:%M").to_string();
                            let date_composed = format!("{date_part} {curr_deadline_hm}");
                            println!("{date_composed}");
                            new_date = Deadline::new(&date_composed);
                        } else if DateTime::is_valid_dmy_format(&date_part) {
                            let curr_deadline_hm = &task.deadline.date.format("%H:%M").to_string();
                            let date_composed = format!("{date_part} {curr_deadline_hm}");
                            new_date = Deadline::new(&date_composed);
                        } else if DateTime::is_valid_hm_format(&date_part) {
                            let curr_deadline_dmy =
                                DateTime::date_dmy(task.deadline.date).to_string();
                            let date_composed = format!("{curr_deadline_dmy} {date_part}");
                            println!("{date_composed} {curr_deadline_dmy}");
                            new_date = Deadline::new(&date_composed);
                        }
                    }

                    match new_date {
                        Ok(deadline) => Postponed(deadline.date),
                        Err(err) => {
                            println!("Error: {}", err);
                            continue;
                        }
                    }
                }
                "expired" => Expired(task.deadline.date.clone()),
                "aborted" => Aborted(DateTime::date_now()),
                _ => {
                    eprintln!("No such status available");
                    continue;
                }
            };
            task.status = new_status;
            break;
        }
    }

    fn edit_deadline(task: &mut Task) {
        let new_value = prompt_field_for_task("deadline");
    }
}

fn edit_task(tasks: &mut Vec<Task>, task_label_number: String) {
    let task_labeled_by = format!("Task {task_label_number}");

    if let Some(task) = tasks.into_iter().find(|x| x.label == task_labeled_by) {
        loop {
            let field_to_edit = prompt_field_for_task("field").to_lowercase().to_string();

            if field_to_edit == "thing" {
                EditTaskConfig::edit_thing(task);
            } else if field_to_edit == "status" {
                EditTaskConfig::edit_status(task);
            } else if field_to_edit == "deadline" {
                EditTaskConfig::edit_deadline(task);
            } else {
                panic!("No such field to edit, you lying son of a bitch!")
            }
            clear_console();

            // marker - edit task

            println!("Updated task:\n{task}");
            tasks_file_manager::save_tasks(tasks);
            break;
        }
    };
}

fn show_completed() {
    ()
}

trait DateTimeFormatter {
    fn is_valid_date_format(date_string: &str) -> bool;
    fn is_valid_dmy_format(date_string: &str) -> bool;
    fn is_valid_hm_format(date_string: &str) -> bool;
    fn parse_string_to_datetime_local(date_string: &str) -> Result<DateTime<Local>, ParseError>;
    fn date_now() -> DateTime<Local>;
    fn date_dmy<'a>(date: DateTime<Local>) -> DelayedFormat<StrftimeItems<'a>>;
    fn date_user_formating(date: DateTime<Local>) -> String;
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
        let format = "%d/%m/%Y";

        match NaiveDateTime::parse_from_str(date_string, format) {
            Ok(_) => true,
            Err(_) => false,
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
        // let dt = dt.format("%d-%m-%Y %H:%M:%S");
        dt
    }

    fn date_dmy<'a>(date: DateTime<Local>) -> DelayedFormat<StrftimeItems<'a>> {
        return date.format("%d/%m/%Y");
    }

    fn date_user_formating(date: DateTime<Local>) -> String {
        return date.format("%d/%m/%Y %H:%M").to_string();
    }
}

pub mod cli_manager {
    use super::*;

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
            "Available actions:\n{}{}{}{}{}",
            "1 | show tasks => Display all tasks (including expired, completed, postponed)\n",
            "2 | add task | add => Add new task (thing, deadline)\n",
            "3 | edit task | edit => Edit task (takes Task id)\n",
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
}

pub mod tasks_file_manager {
    use super::*;

    pub fn save_tasks(tasks: &Vec<Task>) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("./tasks.txt");

        let mut content = String::new();

        tasks.iter().for_each(|task| {
            let task_str = format!(
                "Task {{ thing: {}, status: {:?}, label: {}, deadline: {:?} }}",
                task.thing, task.status, task.label, task.deadline
            );
            // Task { thing: "thing", status: Todo, deadline: Deadline { date: 2023-06-13T12:30:00+02:00 }" }

            content.push_str(&task_str);
            content.push_str("\n");
        });

        match file {
            Ok(mut file) => Ok(file.write_all(content.as_bytes())?),
            Err(err) => self::panic!("{}", err),
        }
    }

    pub fn get_saved_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
        let mut file = OpenOptions::new().read(true).open("./tasks.txt")?;

        let reader = BufReader::new(file);

        let instaces: Vec<Task> = reader
            .lines()
            .map(|line| {
                let line = line.unwrap();
                let parsed = line.split("Task { ");
                let parsed = parsed.filter(|x| x.len() > 0);
                let parsed = parsed.collect::<String>();

                let parsed = parsed
                    .split(",")
                    .map(|x| x.trim().split(": ").collect::<Vec<_>>())
                    .collect::<Vec<Vec<&str>>>();

                let mut instance_fields_hashmap: HashMap<&str, &str> = HashMap::new();

                let parsed = parsed.iter().for_each(|x| {
                    // Spliting instance-like string with ":" delimiter cause some value fields containing ":"
                    // to break to more than size of 2 vector, to be certain we join the rest of the vector, seperating field and value
                    let field = x.get(0).unwrap();
                    // let rest = &x[1..];
                    let value = x.get(1).unwrap();

                    // if value.to_lowercase().to_string().contains("postponed") {
                    //     println!("{field} {value}");
                    // }

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
            status: match instance_fields.remove("status").unwrap() {
                _ => TaskStatus::Todo,
            },
            // status: match instance_fields.remove("status").unwrap() {
            //      "completed" => Completed,
            //     "todo" => Todo,
            //     new_value if new_value.starts_with("postponed ") => {
            //         let date_parts = &new_value.split(" ").collect::<Vec<_>>()[1..];
            //         let mut new_date: Result<Deadline, String> = Err(String::from(
            //             "Could not compose new date for this status for this input; make sure it's in correct format -> postponed 14/06/2023 15:30",
            //         ));

            //         // if second part cannot be parsed into dmy part than we'll try to parse it into the hm part
            //         // if this does not succeed then we'll throw an error

            //         if date_parts.len() == 2 {
            //             let date = date_parts.join(",");
            //             new_date = Deadline::new(&date);
            //         } else if date_parts.len() == 1 {
            //             let date_part = date_parts.join(",");

            //             if date_part == "tomorrow" || date_part == "today" {
            //                 // If the date_parts is size of 2 and this statement matches
            //                 // then we could infere the value of rest of the date which is hours:minues from before date
            //                 let curr_deadline_hm = &task.deadline.date.format("%H:%M").to_string();
            //                 let date_composed = format!("{date_part} {curr_deadline_hm}");
            //                 println!("{date_composed}");
            //                 new_date = Deadline::new(&date_composed);
            //             } else if DateTime::is_valid_dmy_format(&date_part) {
            //                 let curr_deadline_hm = &task.deadline.date.format("%H:%M").to_string();
            //                 let date_composed = format!("{date_part} {curr_deadline_hm}");
            //                 new_date = Deadline::new(&date_composed);
            //             } else if DateTime::is_valid_hm_format(&date_part) {
            //                 let curr_deadline_dmy =
            //                     DateTime::date_dmy(task.deadline.date).to_string();
            //                 let date_composed = format!("{curr_deadline_dmy} {date_part}");
            //                 println!("{date_composed} {curr_deadline_dmy}");
            //                 new_date = Deadline::new(&date_composed);
            //             }
            //         }

            //         match new_date {
            //             Ok(deadline) => Postponed(deadline.date),
            //             Err(err) => {
            //                 println!("Error: {}", err);
            //                 continue;
            //             }
            //         }
            //     }
            //     "expired" => Expired(task.deadline.date.clone()),
            //     "aborted" => Aborted(DateTime::date_now()),
            //     _ => {
            //         eprintln!("No such status available");
            //         continue;
            //     }
            // },
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
            println!("{:?}", slice_of_ids);
            available_ids.extend_from_slice(&slice_of_ids);
            i += (diff - 1) as usize;
        }

        i += 1;
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
