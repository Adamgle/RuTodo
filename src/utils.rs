use crate::tasks_file_manager::{parse_task_from_file, save_tasks};
use std::{collections::HashMap, error::Error, fs::OpenOptions, io::Read};

use crate::Task;
use std::path::PathBuf;

// Accepts absolute file path
pub fn parse_redirected_stream_of_show_tasks(
    tasks: &mut Vec<Task>,
    file_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    println!(
        "Parsing redirected stream of show tasks from file: {:?}",
        file_path
    );

    let mut file = match file_path.canonicalize() {
        Ok(path) => OpenOptions::new().read(true).open(path),
        Err(_) => OpenOptions::new().create(true).write(true).read(true).open(file_path),
    }
    .inspect_err(|e| {
        eprintln!("Error opening file: {}", e);
    })?;

    let mut file_content: String = String::new();

    file.read_to_string(&mut file_content)?;

    file_content = file_content.trim().to_string();

    let mut all_ids = Task::get_all_ids(tasks);

    let mut available_ids = Task::find_available_ids(&mut all_ids).into_iter();

    let mut max_id = match all_ids.into_iter().max() {
        Some(max) => max,
        None => 0,
    };

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

        println!("{instance_entries:?}");

        let task = parse_task_from_file(&mut instance_key_values);

        tasks.push(task);
    }

    if let Err(err) = save_tasks(tasks) {
        eprintln!("Could not save the parsed tasks to file: {err}");
    };

    Ok(())
}
