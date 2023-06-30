// #![allow(unused, unu`sed_variables)]

pub mod utils;

use rutodo::tasks_file_manager;
use rutodo::{spawn_cli_interface, Task};
use std::collections::HashMap;

pub use rutodo::DateTimeFormatter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // std::env::set_var("RUST_BACKTRACE", "1");
    if std::env::args().collect::<Vec<_>>()[1..].is_empty() {
        println!("Welcome in this another useless todoapp that everybody makes and no one uses!");
    };
    let mut tasks: Vec<Task> = Vec::new();

    // Task clone
    let mut tasks_history: HashMap<String, Vec<Task>> = HashMap::new();

    match tasks_file_manager::get_saved_tasks("tasks.txt") {
        Ok(mut instances) => {
            if !instances.is_empty() {
                Task::mark_expired_tasks_as_status_expired(&mut instances);
                instances.into_iter().for_each(|x| tasks.push(x));
            }
        }
        Err(err) => eprintln!("{err}"),
    };

    match tasks_file_manager::get_saved_tasks("tasks_history.txt") {
        Ok(instances) => {
            if !instances.is_empty() {
                for task in instances {
                    tasks_history
                        .entry(task.label.clone())
                        .or_default()
                        .push(task);
                }
            }
        }
        Err(err) => eprintln!("{err}"),
    }

    if let Err(interface_err) = spawn_cli_interface(&mut tasks, &mut tasks_history) {
        eprintln!("{interface_err}")
    }

    Ok(())
}
