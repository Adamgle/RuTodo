#![allow(unused, unused_variables)]

use core::slice;
use std::collections::binary_heap::Iter;

use pointers::{spawn_cli_interface, Task};

use pointers::tasks_file_manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome in this another useless todoapp that everybody makes and no one uses!");
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
