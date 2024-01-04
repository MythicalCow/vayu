use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use colored::Colorize;
use chrono::{Local,Duration};
use std::io::prelude::*;


//struct for the main command.
#[derive(Parser)]
struct Command {
    /// {add,list,done} add: add a new task, list: list all tasks, done: mark a task as done
    command: String,
    /// {add} task description and due date (YYYY-MM-DD or today,yesterday,monday,etc.) separated by a colon. Ex: "vayu add yoga due:today"                         {done} task id. Ex: "vayu done 1"
    #[clap(default_value = "")]
    arg1: String,
}

//struct for a task. there are some weird warnings about this being unused
#[allow(dead_code)]
struct Task {
    description: String,
    due: String,
    done: bool,
    id: i32,
}

fn main() {
    //TASK LIST PARSING FROM LOCAL FILE
    //try opening task list file
    let file = File::open("tasks.txt");
    
    //if the file doesn't exist, create it
    if file.is_err() {
        File::create("tasks.txt").expect("Unable to create file");
    }

    let mut next_id = 0;

    //reads the task list data into a vector of tasks. (this is used by task list and for updating the file after adding a task)
    let mut tasks : Vec<Task> = Vec::new();
    if let Ok(lines) = read_lines("tasks.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(task) = line {
                let task_vec : Vec<&str> = task.split("%").collect();
                let task_id = task_vec[3].to_string().parse::<i32>().unwrap();
                let task = Task {
                    description: task_vec[0].to_string(),
                    due: task_vec[1].to_string(),
                    done: task_vec[2].to_string().parse::<bool>().unwrap(),
                    id: task_id,
                };
                if task_id > next_id {
                    next_id = task_id;
                }
                tasks.push(task);
            }
        }
    }

    //next id to be used is one higher than the highest id in the task list
    next_id += 1;

    //TASK LIST PARSING FROM CLI
    let command : Command = Command::parse();
    if command.command == "add" {
        add_task(&mut tasks, next_id, command.arg1);
        list_tasks(&mut tasks);
    } 
    else if command.command == "list" {
        list_tasks(&mut tasks);
    } 
    else if command.command == "done" {
        remove_task(&mut tasks, command.arg1);
    } 
    else {
        println!("unknown command. use --help to see available commands");
    }

    //write the task list to the file
    let mut file = File::create("tasks.txt").expect("Unable to create file");
    for task in tasks {
        let task_str = format!("{}%{}%{}%{}\n", task.description, task.due, task.done, task.id);
        file.write_all(task_str.as_bytes()).expect("Unable to write data");
    }

}

fn list_tasks(tasks: &mut Vec<Task>) {
    //sort the tasks by due date and store in dtasks
    let dtasks = tasks;
    dtasks.sort_by(|t1, t2| t1.due.cmp(&t2.due));
    //display the tasks
    println!("ID  | Due Date   | Task Description");
    println!("----|------------|-----------------");
    let mut i = 0;
    for task in dtasks {
        if !task.done {
            //make task id a len 3 string pad with spaces
            let mut id = task.id.to_string();
            while id.len() < 4 {
                id.push(' ');
            }
            if i % 2 == 0 {
                println!("{}| {} | {}", id.on_black(), task.due.on_black(), task.description.on_black());
            } else {
                println!("{}| {} | {}", id, task.due, task.description);
            }
            i += 1;
        }
    }

}

fn add_task(tasks: &mut Vec<Task>, next_id: i32, arg1: String) {
    //parse the task description and due date from the arg1 string
    let arg1_vec : Vec<&str> = arg1.split(":").collect();
    if arg1_vec.len() != 2 {
        println!("invalid usage of add. use --help to see usage");
        return;
    }
    let mut task_desc = arg1_vec[0].to_string();
    task_desc.truncate(task_desc.len() - 3);
    let mut due_date = arg1_vec[1].to_string();
    due_date.retain(|c| !r#"\n"#.contains(c));

    //if the due date is "today" or "tomorrow" or "yesterday" or a day of the week, set the due date to the appropriate date
    //we use DateTime<Local> to get the current date and time
    let now = Local::now();
    if due_date == "today" {
        due_date = now.format("%Y-%m-%d").to_string();
    } else if due_date == "tomorrow" {
        due_date = (now + Duration::days(1)).format("%Y-%m-%d").to_string();
    } else if due_date == "yesterday" {
        due_date = (now - Duration::days(1)).format("%Y-%m-%d").to_string();
    } else if due_date == "monday" {
        due_date = (now + Duration::days(1)).format("%Y-%m-%d").to_string();
    } else if due_date == "tuesday" {
        due_date = (now + Duration::days(2)).format("%Y-%m-%d").to_string();
    } else if due_date == "wednesday" {
        due_date = (now + Duration::days(3)).format("%Y-%m-%d").to_string();
    } else if due_date == "thursday" {
        due_date = (now + Duration::days(4)).format("%Y-%m-%d").to_string();
    } else if due_date == "friday" {
        due_date = (now + Duration::days(5)).format("%Y-%m-%d").to_string();
    } else if due_date == "saturday" {
        due_date = (now + Duration::days(6)).format("%Y-%m-%d").to_string();
    } else if due_date == "sunday" {
        due_date = (now + Duration::days(7)).format("%Y-%m-%d").to_string();
    }


    //create a new task and add it to the task list
    let task = Task {
        description: task_desc,
        due: due_date,
        done: false,
        id: next_id,
    };
    tasks.push(task);
    println!("task added with id {}", next_id.to_string().green())
}

fn remove_task(tasks: &mut Vec<Task>, arg1: String) {
    //parse the task id from the arg1 string
    let task_id = arg1.parse::<i32>().unwrap();
    //find the task with the given id and remove it from the task list
    let mut index = 0;
    for task in &mut tasks.iter() {
        if task.id == task_id {
            tasks.swap_remove(index);
            println!("task {} done", task_id.to_string().green());
            return;
        }
        index += 1;
    }
    println!("task with id {} not found", task_id.to_string().red());
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
}
