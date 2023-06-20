use std::{collections::HashMap, error::Error, fs::OpenOptions, io::Read};

use crate::tasks_file_manager::{parse_task_from_file, save_tasks};
use crate::Task;

// Accepts absolute file path
pub fn parse_redirected_stream_of_show_tasks(
    tasks: &mut Vec<Task>,
    file_path: std::path::PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;

    let mut file_content: String = String::new();

    file.read_to_string(&mut file_content)?;

    file_content = file_content.trim().to_string();

    file_content.split("\n\r").for_each(|line| {
        let mut instance_key_values = HashMap::new();

        let line = line.replace("\n", "").trim().to_string();
        let line = line
            .split("\r")
            .enumerate()
            .map(|(idx, e)| match idx {
                0 => format!("label: {}", e),
                _ => e.to_string(),
            })
            .collect::<Vec<String>>();

        line.iter().for_each(|x| {
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
        tasks.push(task);
    });

    if let Err(err) = save_tasks(&tasks) {
        eprint!("Could not save the parsed tasks to file: {err}");
    };

    Ok(())
}
