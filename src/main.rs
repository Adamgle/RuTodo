// #![allow(unused, unused_variables)]

pub mod utils;

use rutodo::{spawn_cli_interface, Task};

pub use rutodo::DateTimeFormatter;

use rutodo::tasks_file_manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().collect::<Vec<_>>()[1..].is_empty() {
        println!("Welcome in this another useless todoapp that everybody makes and no one uses!");
    };
    let mut tasks: Vec<Task> = Vec::new();

    match tasks_file_manager::get_saved_tasks() {
        Ok(instances) => {
            if !instances.is_empty() {
                instances.into_iter().for_each(|x| tasks.push(x))
            }
        }
        Err(err) => eprintln!("{err}"),
    };

    if let Err(interface_err) = spawn_cli_interface(&mut tasks) {
        eprintln!("{interface_err}")
    }

    Ok(())
}
