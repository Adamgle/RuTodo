# RuTodo

A command-line todo application written in Rust for task management with deadlines and status tracking.

# Motivation

Created to explore and grasp some experience writing Rust.

## What is RuTodo?

RuTodo is a terminal-based todo application that helps you manage tasks with deadlines. It automatically tracks task expiration, supports task history, and provides powerful filtering and sorting capabilities through a CLI interface with available documentation.

## Features

- **Task Management**: Add, edit, delete, and view tasks
- **Deadline Support**: Set deadlines with flexible date formats
- **Status Tracking**: Track tasks as Todo, Completed, Postponed, Expired, or Aborted
- **Advanced Filtering**: Filter tasks by status, deadline, or description
- **Sorting Options**: Sort by deadline, alphabetically, or by date
- **Task History**: Keep track of task modifications
- **Auto-Save**: Automatic task persistence to file
- **Cross-Platform**: Works on Windows and Unix-like systems
- **Export**: Redirect task output to files
- **Import**: Allow to parse redirect stream of tasks back into the program.
- **Filter Aggregation**: Each filter can be bundled producing a result that is carried over the next filter.

## Date Format Support

Multiple flexible date formats supported:

- `10/06/2023 12:30` - Full date and time
- `10/06/2023` - Date only (defaults to 00:00)
- `tomorrow 12:30` - Relative date with time
- `today`, `tomorrow`, `next` - Relative dates
- `12:30` - Time only (uses today's date)

## Basic Usage

### Interactive CLI Mode

```bash
cargo run
```

### Command Line Arguments

```bash
# Show all tasks
cargo run -- --show-tasks

# Show tasks with filtering
cargo run -- --show-tasks --status completed
cargo run -- --show-tasks --deadline tomorrow
cargo run -- --show-tasks --thing "buy"

# Parse tasks from file
cargo run -- --parse tasks/real-stuff.txt
```

### Available Commands in Interactive Mode

1. **Show Tasks**: `1` or `show tasks`
2. **Add Task**: `2` or `add task`
3. **Edit Task**: `3 <task_id>` or `edit <task_id> [--field]`
4. **Delete Task**: `4 <task_id>` or `delete <task_id>` or `delete all`
5. **Help**: `help`
6. **Clear Console**: `cls`
7. **Exit**: `exit`

## Quick Documentation

Documentation is available for each exposed command with a `--help` switch.

### Task Status Types

- **Todo**: Default status for new tasks
- **Completed**: Manually marked as done
- **Postponed**: Delayed to a new date
- **Expired**: Past deadline (auto-marked)
- **Aborted**: Cancelled task

### Filtering Examples

```bash
show tasks --thing "buy"          # Filter by description
show tasks --status completed     # Filter by status
show tasks --deadline tomorrow    # Filter by deadline
show tasks --date -gt today       # Show future tasks
show tasks --alphabetical         # Sort alphabetically
cargo run -- --show-tasks --date -gt tomorrow --status postponed --alphabetical --redirect real-thing.txt # Filter Aggregation. Show tasks with deadline greater than tomorrow that are were postponed, got alphabetically sorted with output redirect to file.
```

## Known Issues

- Task ID management could be improved for better sequential numbering
- File parsing may not handle all edge cases properly
- Date parsing errors could provide more user-friendly messages
- Large task files may impact performance
- Limited undo functionality for task modifications

### Code clarity

- The code gets deeply nested and might trigger the trauma from Javascript.
- Poor code separation

## Dependencies

- `chrono` - Date and time handling
- `whoami` - System user information

## File Storage

Tasks are automatically saved to:

- Windows: `%SystemDrive%/Users/%USERNAME%/documents/rust-todo/tasks.txt`
- Other: Current working directory

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run
```
