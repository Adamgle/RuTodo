use crate::tasks_file_manager::{parse_task_from_file, save_tasks};
use std::io::Write;
use std::{collections::HashMap, error::Error, fs::OpenOptions, io::Read};

use crate::DateTimeFormatter;
use crate::Task;
use chrono::DateTime;
use std::path::PathBuf;

// Accepts absolute file path
pub fn parse_redirected_stream_of_show_tasks(
    tasks: &mut Vec<Task>,
    file_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;
    let mut log_file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(r"C:\Dev\Rust\rutodo\logs.txt")?;

    let mut file_content: String = String::new();

    file.read_to_string(&mut file_content)?;

    file_content = file_content.trim().to_string();

    let mut all_ids = Task::get_all_ids(tasks);

    let mut available_ids = Task::find_available_ids(&mut all_ids).into_iter();

    let mut max_id = match all_ids.into_iter().max() {
        Some(max) => max,
        None => 0,
    };

    log_file
        .write_fmt(format_args!("Begin log: {}\n\n", DateTime::date_now()))
        .expect("Could not write to file log");

    let mut key_value_fields_vec: Vec<Vec<String>> = vec![];

    let mut split_from = 0;
    let lines = file_content
        .lines()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    lines.iter().enumerate().for_each(|(idx, line)| {
        if line == "" {
            if split_from == 0 {
                key_value_fields_vec.push(lines[split_from..idx].to_vec());
            } else {
                key_value_fields_vec.push(lines[split_from + 1..idx].to_vec());
            }
            split_from = idx;
        } else if idx == lines.len() - 1 {
            key_value_fields_vec.push(lines[split_from + 1..].to_vec());
        }
    });

    let mut instance_key_values: HashMap<String, String> = HashMap::new();

    for entries in key_value_fields_vec {
        let next_id = match available_ids.next() {
            Some(id) => id,
            None => {
                max_id += 1;
                max_id
            }
        };

        let instance_entries = entries
            .into_iter()
            .enumerate()
            .map(|(idx, e)| match idx {
                0 => format!("label: Task {next_id}"),
                _ => e.to_string(),
            })
            .collect::<Vec<String>>();

        instance_entries.iter().for_each(|x| {
            if let Some((key, value)) = x.split_once(":") {
                let key = key.trim().to_lowercase();
                let value = if key == "thing" {
                    format!("\"{}\"", value.trim())
                } else {
                    value.trim().to_string()
                };
                instance_key_values.insert(key.trim().to_lowercase(), value);
            }
        });

        let task = parse_task_from_file(&mut instance_key_values);

        log_file
            .write_fmt(format_args!("{}\n", task))
            .expect("Could not write to file log");

        tasks.push(task);
    }

    if let Err(err) = save_tasks(tasks) {
        eprintln!("Could not save the parsed tasks to file: {err}");
    };

    Ok(())
}
