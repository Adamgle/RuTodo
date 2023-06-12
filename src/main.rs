#![allow(unused, unused_variables)]

use chrono::format::{strftime::StrftimeItems, DelayedFormat, ParseError};
use chrono::prelude::*;
use chrono::Duration;
use core::panic;
use std::collections::HashMap;
use std::error::Error;
// use std::ffi::IntoStringError;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::time::SystemTime;

#[derive(Debug)]
struct Task {
    thing: String,
    status: TaskStatus,
    deadline: Deadline,
}

// task: asdasd
// status: Todo,
// deadline: day/month/year -> 10/6/2023

#[derive(Debug)]
struct Deadline {
    // isPostponed: bool,
    date: DateTime<Local>,
}

#[derive(Debug)]
enum TaskStatus {
    Completed,
    Todo,
    PostponedTo(DateTime<Local>),
    Expired(DateTime<Local>),
}

// enum DeadlineDayKeywords {}

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
                        let day_month_year = Self::date_hms(Self::date_now());

                        Ok(day_month_year
                            .to_string()
                            .split("/")
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>())
                    }
                    "tomorrow" => {
                        let day_month_year = Self::date_hms(Self::date_now() + Duration::days(1));

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

        match parse_string_to_datetime_local(&date.join(" ").trim()) {
            Ok(date) => Ok(Self { date }),
            Err(err) => Err(format!("Error occurred while parsing the date. Make sure it's in the right format\nMessage: {}", err)),
        }
    }

    // returns d/m/y format
    fn date_now() -> DateTime<Local> {
        let curr_time = SystemTime::now();
        let dt: DateTime<Local> = curr_time.clone().into();
        // let dt = dt.format("%d-%m-%Y %H:%M:%S");
        dt
    }

    fn date_hms<'a>(date: DateTime<Local>) -> DelayedFormat<StrftimeItems<'a>> {
        return date.format("%d/%m/%Y");
    }

    fn date_user_formating(date: DateTime<Local>) -> String {
        return date.format("%d/%m/%Y %H:%M").to_string();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome in this another useless todoapp that everybody makes and no one uses!");
    let mut tasks: Vec<Task> = Vec::new();

    match get_saved_tasks() {
        Ok(instances) => {
            if !instances.is_empty() {
                instances.into_iter().for_each(|x| tasks.push(x))
            }
        }
        Err(err) => eprintln!("{err}"),
    };

    spawn_cli_interface(&mut tasks);

    Ok(())
}

fn clear_console_and_display_help() {
    clear_console();
    show_user_actions();
}

fn spawn_cli_interface(tasks: &mut Vec<Task>) -> Result<(), String> {
    show_user_actions();

    loop {
        let mut action = String::new();

        print!("promp: ");
        io::stdout().flush().expect("Failed to flush stdout");

        io::stdin()
            .read_line(&mut action)
            .expect("Unable to write to the writtable buffer");

        // clear_console();

        match action.trim() {
            "1" | "show tasks" => show_tasks(tasks),
            "2" | "add task" | "add" => add_new_task(tasks),
            "exit" => std::process::exit(0),
            "help" => show_user_actions(),
            _ => eprintln!("Unrecognized program action"),
        };
    }
}

fn prompt_field_for_task(field_name: &str) -> String {
    let mut input = String::new();

    print!("{field_name}: ");

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
                    continue; // Continue the outer loop
                }
            };
            break;
        }

        let task = Task {
            thing,
            status: TaskStatus::Todo,
            deadline: parsed_deadline,
        };

        println!(
            "Task successfully added:\nTask {{ thing: {}, status: {:?}, deadline: {} }}",
            task.thing,
            task.status,
            Deadline::date_hms(task.deadline.date)
        );

        tasks.push(task);

        save_tasks(&tasks);
    }
}

fn edit_task() {
    ()
}

fn show_tasks(tasks: &Vec<Task>) {
    if tasks.len() == 0 {
        println!("No available tasks");
        return ();
    }

    let mut formated = String::new();

    for (idx, task) in tasks.iter().enumerate() {
        formated.push_str(format!("Task {}\n", idx + 1).as_str());
        formated.push_str(
            format!(
                "Thing: {}\nstatus: {:?}\ndeadline: {}\n{}",
                task.thing,
                task.status,
                Deadline::date_user_formating(task.deadline.date),
                if tasks.len() > 1 { "\n" } else { "" }
            )
            .as_str(),
        )
    }

    println!("{formated}")
}

fn show_user_actions() {
    // clear_console();
    println!(
        "Available actions:\n{}{}{}{}",
        "1 | show tasks => Display all tasks (including expired, completed, postponed)\n",
        "2 | add task | add => Add new task (thing, deadline)\n",
        "help => Display this help message\n",
        "exit => Terminates current procces\n",
    )
}

fn clear_console() {
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

fn show_completed() {
    ()
}

fn is_valid_date_format(date_string: &str) -> bool {
    let format = "%d/%m/%Y %H:%M";
    match NaiveDateTime::parse_from_str(date_string, format) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn parse_string_to_datetime_local(date_string: &str) -> Result<DateTime<Local>, ParseError> {
    // Define the format of the input string
    let format = "%d/%m/%Y %H:%M";

    // if !is_valid_date_format(date_string) {
    //     return Err("Invalid date format");
    // }

    // Parse the string into a NaiveDateTime
    let naive_datetime = NaiveDateTime::parse_from_str(date_string, format)?;

    // Convert the NaiveDateTime to a DateTime<Local>
    let datetime_local = Local
        .from_local_datetime(&naive_datetime)
        .single()
        .expect("Ambiguous or non-existent local time");

    Ok(datetime_local)
}

fn get_saved_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
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

fn parse_task_from_file(instance_fields: &mut HashMap<&str, &str>) -> Task {
    Task {
        thing: instance_fields.remove("thing").unwrap().to_string(),
        status: match instance_fields.remove("status").unwrap() {
            _ => TaskStatus::Todo,
        },
        deadline: Deadline {
            date: match DateTime::parse_from_rfc3339(instance_fields.remove("deadline").unwrap()) {
                Ok(date) => date.with_timezone(&Local),
                Err(err) => panic!("{err}"),
            },
        },
    }
}

fn save_tasks(tasks: &Vec<Task>) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .write(true)
        // .append(true)
        .create(true)
        .open("./tasks.txt");
    // let file_opened = OpenOptions::new().write(true).open("tasks.txt");

    let mut content = String::new();

    tasks.iter().for_each(|task| {
        // Task { thing: "asdsa", status: Todo, deadline: Deadline { date: 2023-06-12T12:30:00+02:00 } }
        let task_str = format!(
            "Task {{ thing: {}, status: {:?}, deadline: {:?} }}",
            task.thing, task.status, task.deadline
        );
        // let string_variable = String::from(task_str);
        // let raw_string = r#""#.to_owned() + &task_str + r#" "#;
        // println!("{raw_string}");
        content.push_str(&task_str);
        content.push_str("\n");
    });

    match file {
        Ok(mut file) => Ok(file.write_all(content.as_bytes())?),
        Err(err) => panic!("{}", err),
    }
}
