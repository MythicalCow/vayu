extern crate clap;
extern crate colored;
extern crate chrono;
extern crate indicatif;

//tui
use std::io::{self, stdout, BufRead};
use crossterm::{
    event::{self, Event as UIEvent, KeyCode},
    ExecutableCommand,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{prelude::*, widgets::*};
use clap::Parser;
use std::fs::File;
use std::path::Path;
use chrono::{Local,Duration};
use std::io::prelude::*;
//use chrono::Datelike;
use indicatif::ProgressBar;

//rewrite for CLI parser using subcommand feature
//add, list, done, pomo, eadd, elist, eids, erem
#[derive(Parser)]
enum SubComm {
    Add{
        /// task description and due date (YYYY-MM-DD or today,yesterday,monday,etc.) separated by a colon. Ex: "vayu add yoga due:today"
        arg1: String,
    },
    Auto{
        /// auto generate a task. Ex: vayu auto "test at end of month"
        arg1: String,
    },
    List{
    },
    Done{
        /// task id. Ex: "vayu done 1"
        arg1: String,
    },
    Pomo{
        /// number of work sessions
        arg1: String,
        /// length of work session in minutes
        arg2: String,
        /// length of break session in minutes
        arg3: String,
    },
    Eadd{
        /// event description
        arg1: String,
        /// event start time (H:MMam or H:MMpm)
        arg2: String,
        /// event end time (H:MMam or H:MMpm)
        arg3: String,
        /// event repeat (day1,day2,day3,day4,day5,day6,day7 where dayi is a day of the week) or (YYYY-MM-DD) or (everyday,weekday,weekend)
        arg4: String,
    },
    Elist{
    },
    Eids{
    },
    Erem{
        /// event id to remove
        arg1: String,
    },
}

//struct for the main command.
#[derive(Parser)]
struct Arguments {
    #[clap(default_value = "")]
    command: String,
    #[clap(default_value = "")]
    arg1: String,
    #[clap(default_value = "")]
    arg2: String,
    #[clap(default_value = "")]
    arg3: String,
    #[clap(default_value = "")]
    arg4: String,
}

//struct for a task. there are some weird warnings about this being unused
#[allow(dead_code)]
#[derive(Clone)]
struct Task {
    description: String,
    due: String,
    done: bool,
    id: i32,
}

#[allow(dead_code)]
#[derive(Clone)]
struct Event {
    description: String,
    start: String,
    end: String,
    repeat: String,
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

    let file2 = File::open("events.txt");
    //if the file doesn't exist, create it
    if file2.is_err() {
        File::create("events.txt").expect("Unable to create file");
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

    let mut next_event_id = 0;

    //reads the event list data into a vector of events. (this is used by event list and for updating the file after adding an event)
    let mut events : Vec<Event> = Vec::new();
    if let Ok(lines) = read_lines("events.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(event) = line {
                let event_vec : Vec<&str> = event.split("%").collect();
                let event_id = event_vec[4].to_string().parse::<i32>().unwrap();
                let event = Event {
                    description: event_vec[0].to_string(),
                    start: event_vec[1].to_string(),
                    end: event_vec[2].to_string(),
                    repeat: event_vec[3].to_string(),
                    id: event_id,
                };
                if event_id > next_event_id {
                    next_event_id = event_id;
                }
                events.push(event);
            }
        }
    }

    //next id to be used is one higher than the highest id in the event list
    next_event_id += 1;



    //CLI PARSING
    let matches = Arguments::parse();
    match matches.command.as_str() {
        //use the subcommands so that --help works for each subcommand
        //we want to see the internal comments for each subcommand so we can't use the macro
        "add" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Add{arg1} => {
                    add_task(&mut tasks, next_id, arg1);
                },
                _ => {
                    println!("invalid usage of add. use --help to see usage");
                }
            }
        },
        "auto" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Auto{arg1} => {
                    add_auto(&mut tasks, next_id, arg1);
                },
                _ => {
                    println!("invalid usage of auto. use --help to see usage");
                }
            }
        }
        "list" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::List{} => {
                    list_tasks(&mut tasks);
                },
                _ => {
                    println!("invalid usage of list. use --help to see usage");
                }
            }
        },
        "done" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Done{arg1} => {
                    remove_task(&mut tasks, arg1);
                },
                _ => {
                    println!("invalid usage of done. use --help to see usage");
                }
            }
        },
        "pomo" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Pomo{arg1, arg2, arg3} => {
                    pomodoro(arg1, arg2, arg3);
                },
                _ => {
                    println!("invalid usage of pomo. use --help to see usage");
                }
            }
        },
        "eadd" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Eadd{arg1, arg2, arg3, arg4} => {
                    add_event(&mut events, arg1, arg2, arg3, arg4, next_event_id);
                },
                _ => {
                    println!("invalid usage of eadd. use --help to see usage");
                }
            }
        },
        "elist" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Elist{} => {
                    daily_agenda(&mut events);
                },
                _ => {
                    println!("invalid usage of elist. use --help to see usage");
                }
            }
        },
        "eids" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Eids{} => {
                    list_event_ids(&mut events);
                },
                _ => {
                    println!("invalid usage of eids. use --help to see usage");
                }
            }
        },
        "erem" => {
            let submatches = SubComm::parse();
            match submatches {
                SubComm::Erem{arg1} => {
                    remove_event(&mut events, arg1);
                },
                _ => {
                    println!("invalid usage of erem. use --help to see usage");
                }
            }
        },
        "" => {
            //if no command is given, run the vayu ui
            vayu_ui(&mut tasks, &mut events).expect("error");
        },
        _ => {
            println!("invalid command. use --help to see usage");
        }



    }
    //write the task list to the file
    let mut file = File::create("tasks.txt").expect("Unable to create file");
    for task in tasks {
        let task_str = format!("{}%{}%{}%{}\n", task.description, task.due, task.done, task.id);
        file.write_all(task_str.as_bytes()).expect("Unable to write data");
    }

    //write the event list to the file
    let mut file = File::create("events.txt").expect("Unable to create file");
    for event in events {
        let event_str = format!("{}%{}%{}%{}%{}\n", event.description, event.start, event.end, event.repeat, event.id);
        file.write_all(event_str.as_bytes()).expect("Unable to write data");
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
            //if due date is today use red text
            if task.due == Local::now().format("%Y-%m-%d").to_string() {
                println!("{}| {} | {}", id, task.due, task.description);
            }
            else if i % 2 == 0 {
                println!("{}| {} | {}", id, task.due, task.description);
            } 
            else {
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
    //remove \n from the end of due_date
    due_date = due_date.replace("\n", "");
    println!("due date: {}", due_date);
    //if the due date is "today" or "tomorrow" or "yesterday" or a day of the week, set the due date to the appropriate date
    //we use DateTime<Local> to get the current date and time
    let now = Local::now();
    //calculate number of days from 
    if due_date == "today" {
        due_date = now.format("%Y-%m-%d").to_string();
    } 
    else if due_date == "tomorrow" {
        due_date = (now + Duration::days(1)).format("%Y-%m-%d").to_string();
    } 
    else if due_date == "yesterday" {
        due_date = (now - Duration::days(1)).format("%Y-%m-%d").to_string();
    }
    else if due_date == "sunday" || due_date == "monday" || due_date == "tuesday" || due_date == "wednesday" || due_date == "thursday" || due_date == "friday" || due_date == "saturday"{
        let mut duecp = due_date.clone();
        let capsdate = duecp.remove(0).to_uppercase().to_string() + &duecp;
        let mut day = now;
        while day.format("%A").to_string() != capsdate{
            day = day + Duration::days(1);
        }
        if now.format("%A").to_string() == capsdate{
            day = day + Duration::days(7);
        }
        due_date = day.format("%Y-%m-%d").to_string();
    }
    else{
        println!("invalid due date. use YYYY-MM-DD or today, tomorrow, yesterday, or a day of the week");
    }



    //create a new task and add it to the task list
    let task = Task {
        description: task_desc,
        due: due_date,
        done: false,
        id: next_id,
    };
    tasks.push(task);
    println!("task added with id {}", next_id)
}

fn add_auto(tasks: &mut Vec<Task>, next_id: i32, arg1: String) {
    //if "end" and "month" are in the arg1 string, add a task for the end of the month
    if arg1.contains("end") && arg1.contains("month") {
        //get the current date
        let mut now = Local::now();
        //while next day is in the same month, add one day
        while(now + Duration::days(1)).format("%m").to_string() == now.format("%m").to_string() {
            now = now + Duration::days(1);
        }
        //set the due date to now in the format YYYY-MM-DD
        let last_day = now.format("%Y-%m-%d").to_string();

        //remove end and month from the arg1 string
        let mut task_desc = arg1.replace("end", "");
        task_desc = task_desc.replace("month", "");
        //while the task description has a last word of "by", "at", "of" or "on",  remove the last word
        let mut task_desc_vec : Vec<&str> = task_desc.split(" ").collect();
        //remove all "" entries
        task_desc_vec.retain(|&x| x != "");
        let mut last_word: &str = task_desc_vec[task_desc_vec.len()-1];  
        let mut desc_temp: String = task_desc.clone();
        //print last_word
        while last_word == "by" || last_word == "at" || last_word == "of" || last_word == "on" || last_word == "the" || last_word == "this" {
            task_desc_vec.pop();
            desc_temp = task_desc_vec.join(" ");
            last_word = task_desc_vec[task_desc_vec.len()-1];
        }

        //print the due date and task description and ask for confirmation
        println!("Auto Generated Task with due date: {}. Is it ok (type yes/y)? ", last_day);
        stdout().flush().unwrap();
        //get user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("error");
        //if the user input is not "y" or "yes", return
        if input.to_lowercase().trim() != "y" && input.to_lowercase().trim() != "yes" {
            println!("task not added");
            return;
        }
        else{
            let task = Task {
                description: desc_temp,
                due: last_day,
                done: false,
                id: next_id,
            };
            tasks.push(task);
            println!("task added with id {}", next_id);
        }
    }
    //repeat for end of year
    else if arg1.contains("end") && arg1.contains("year") {
        //get the current date
        let mut now = Local::now();
        //while next day is in the same year, add one day
        while(now + Duration::days(1)).format("%Y").to_string() == now.format("%Y").to_string() {
            now = now + Duration::days(1);
        }
        //set the due date to now in the format YYYY-MM-DD
        let last_day = now.format("%Y-%m-%d").to_string();

        //remove end and year from the arg1 string
        let mut task_desc = arg1.replace("end", "");
        task_desc = task_desc.replace("year", "");
        //while the task description has a last word of "by", "at", "of" or "on",  remove the last word
        let mut task_desc_vec : Vec<&str> = task_desc.split(" ").collect();
        task_desc_vec.retain(|&x| x != "");
        let mut last_word: &str = task_desc_vec[task_desc_vec.len()-1];  
        let mut desc_temp: String = task_desc.clone();
        //print last_word
        while last_word == "by" || last_word == "at" || last_word == "of" || last_word == "on" || last_word == "the" || last_word == "this" {
            task_desc_vec.pop();
            desc_temp = task_desc_vec.join(" ");
            last_word = task_desc_vec[task_desc_vec.len()-1];
        }

        //print the due date and task description and ask for confirmation
        println!("Auto Generated Task with due date: {}. Is it ok (type yes/y)? ", last_day);
        stdout().flush().unwrap();
        //get user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("error");
        //if the user input is not "y" or "yes", return
        if input.to_lowercase().trim() != "y" && input.to_lowercase().trim() != "yes" {
            println!("task not added");
            return;
        }
        else{
            let task = Task {
                description: desc_temp,
                due: last_day,
                done: false,
                id: next_id,
            };
            tasks.push(task);
            println!("task added with id {}", next_id);
        }
    }
    //repeat for end of week
    else if arg1.contains("end") && arg1.contains("week") {
        //get the current date
        let mut now = Local::now();
        //go until day is sunday
        while(now + Duration::days(1)).format("%A").to_string() != "Sunday".to_string() {
            now = now + Duration::days(1);
        }
        //set the due date to now in the format YYYY-MM-DD
        let last_day = now.format("%Y-%m-%d").to_string();

        //remove end and week from the arg1 string
        let mut task_desc = arg1.replace("end", "");
        task_desc = task_desc.replace("week", "");
        //while the task description has a last word of "by", "at", "of" or "on",  remove the last word
        let mut task_desc_vec : Vec<&str> = task_desc.split(" ").collect();
        task_desc_vec.retain(|&x| x != "");
        let mut last_word: &str = task_desc_vec[task_desc_vec.len()-1];  
        let mut desc_temp: String = task_desc.clone();
        //print last_word
        while last_word == "by" || last_word == "at" || last_word == "of" || last_word == "on" || last_word == "the" || last_word == "this" {
            task_desc_vec.pop();
            desc_temp = task_desc_vec.join(" ");
            last_word = task_desc_vec[task_desc_vec.len()-1];
        }

        //print the due date and task description and ask for confirmation
        println!("Auto Generated Task with due date: {}. Is it ok (type yes/y)? ", last_day);
        stdout().flush().unwrap();
        //get user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("error");
        //if the user input is not "y" or "yes", return
        if input.to_lowercase().trim() != "y" && input.to_lowercase().trim() != "yes" {
            println!("task not added");
            return;
        }
        else{
            let task = Task {
                description: desc_temp,
                due: last_day,
                done: false,
                id: next_id,
            };
            tasks.push(task);
            println!("task added with id {}", next_id);
        }
    }
    //check for days of the week shorthands
    else if arg1.contains(" mon") || arg1.contains(" tue") || arg1.contains(" wed") || arg1.contains(" thu") || arg1.contains(" fri") || arg1.contains(" sat") || arg1.contains(" sun") {
        //get the current date
        let mut now = Local::now();
        //if " mon" due date is set to monday
        if arg1.contains(" mon") {
            while now.format("%A").to_string() != "Monday".to_string() {
                now = now + Duration::days(1);
            }
        }
        //if " tue" due date is set to tuesday
        if arg1.contains(" tue") {
            while now.format("%A").to_string() != "Tuesday".to_string() {
                now = now + Duration::days(1);
            }
        }
        //if " wed" due date is set to wednesday
        if arg1.contains(" wed") {
            while now.format("%A").to_string() != "Wednesday".to_string() {
                now = now + Duration::days(1);
            }
        }
        //if " thu" due date is set to thursday
        if arg1.contains(" thu") {
            while now.format("%A").to_string() != "Thursday".to_string() {
                now = now + Duration::days(1);
            }
        }
        //if " fri" due date is set to friday
        if arg1.contains(" fri") {
            while now.format("%A").to_string() != "Friday".to_string() {
                now = now + Duration::days(1);
            }
        }
        //if " sat" due date is set to saturday
        if arg1.contains(" sat") {
            while now.format("%A").to_string() != "Saturday".to_string() {
                now = now + Duration::days(1);
            }
        }
        //if " sun" due date is set to sunday
        if arg1.contains(" sun") {
            while now.format("%A").to_string() != "Sunday".to_string() {
                now = now + Duration::days(1);
            }
        }

        //set the due date to now in the format YYYY-MM-DD
        let last_day = now.format("%Y-%m-%d").to_string();
        let mut task_desc = arg1;
        //remove the day of the week from the arg1 string (try removing anything from mon to monday)
        let mut remmon: String = "monday".to_string();
        while remmon.len() > 2{
            if task_desc.contains(&remmon){
                task_desc = task_desc.replace(&remmon, "");
                
            }
            remmon.pop();
        }
        let mut remtues: String = "tuesday".to_string();
        while remtues.len() > 2{
            if task_desc.contains(&remtues){
                task_desc = task_desc.replace(&remtues, "");
                
            }
            remtues.pop();
        }
        let mut remwed: String = "wednesday".to_string();
        while remwed.len() > 2{
            if task_desc.contains(&remwed){
                task_desc = task_desc.replace(&remwed, "");
                
            }
            remwed.pop();
        }
        let mut remthurs: String = "thursday".to_string();
        while remthurs.len() > 2{
            if task_desc.contains(&remthurs){
                task_desc = task_desc.replace(&remthurs, "");
                
            }
            remthurs.pop();
        }
        let mut remfri: String = "friday".to_string();
        while remfri.len() > 2{
            if task_desc.contains(&remfri){
                task_desc = task_desc.replace(&remfri, "");
                
            }
            remfri.pop();
        }
        let mut remsat: String = "saturday".to_string();
        while remsat.len() > 2{
            if task_desc.contains(&remsat){
                task_desc = task_desc.replace(&remsat, "");
                
            }
            remsat.pop();
        }
        let mut remsun: String = "sunday".to_string();
        while remsun.len() > 2{
            if task_desc.contains(&remsun){
                task_desc = task_desc.replace(&remsun, "");
                
            }
            remsun.pop();
        }

        

        //while the task description has a last word of "by", "at", "of" or "on",  remove the last word
        
        let mut task_desc_vec : Vec<&str> = task_desc.split(" ").collect();
        task_desc_vec.retain(|&x| x != "");
        let mut last_word: &str = task_desc_vec[task_desc_vec.len()-1];  
        let mut desc_temp: String = task_desc.clone();
        //print last_word
        while last_word == "by" || last_word == "at" || last_word == "of" || last_word == "on" || last_word == "the" || last_word == "this" {
            task_desc_vec.pop();
            desc_temp = task_desc_vec.join(" ");
            last_word = task_desc_vec[task_desc_vec.len()-1];
        }

        //print the due date and task description and ask for confirmation
        println!("Auto Generated Task with due date: {}. Is it ok (type yes/y)? ", last_day);
        stdout().flush().unwrap();
        //get user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("error");
        //if the user input is not "y" or "yes", return
        if input.to_lowercase().trim() != "y" && input.to_lowercase().trim() != "yes" {
            println!("task not added");
        }
        else{
            let task = Task {
                description: desc_temp,
                due: last_day,
                done: false,
                id: next_id,
            };
            tasks.push(task);
            println!("task added with id {}", next_id);
        }
    }     
}

fn remove_task(tasks: &mut Vec<Task>, arg1: String) {
    //parse the task id from the arg1 string
    let task_id = arg1.parse::<i32>().unwrap();
    //find the task with the given id and remove it from the task list
    let mut index = 0;
    for task in &mut tasks.iter() {
        if task.id == task_id {
            tasks.swap_remove(index);
            println!("task {} done", task_id);
            return;
        }
        index += 1;
    }
    println!("task with id {} not found", task_id);
}

fn pomodoro(arg1: String, arg2: String, arg3: String){
    //we will use the chrono crate to get the current time and to calculate the time remaining
    //we will use indicatif to display a progress bar
    //if any of the arguments are empty, throw error
    if arg1 == "" || arg2 == "" || arg3 == "" {
        println!("invalid usage of pomo. use --help to see usage");
        return;
    }
    let iterations = arg1.parse::<i32>().unwrap();
    let work_time = arg2.parse::<i32>().unwrap();
    let break_time = arg3.parse::<i32>().unwrap();

    
    for i in 0..iterations {
        //reupdate end times and start times
        let mut now = Local::now();
        let work_end = now + Duration::minutes(work_time.into());
        //work session
        let mut pb = ProgressBar::new(work_time as u64);
        //make the progress bar smaller
        pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{bar:40.green/red} {msg}").expect("error"));
        //pb.set_message(format!("{} minutes remaining", work_time));
        //figure out number of seconds in one increment (divide seconds by number of increments (40))
        let mut increment = (work_time * 60) as f64 / 40.0;
        for j in 0..(work_time * 60) {
            let remaining = work_end - Local::now();
            //minutes and seconds
            if j % increment as i32 == 0 {
                pb.inc(1);
            }
            pb.set_message(format!("{} minutes {} seconds remaining... work session {} of {}", remaining.num_minutes(), remaining.num_seconds() % 60, (i+1), iterations));
            std::thread::sleep(std::time::Duration::from_secs(1));
            
        }
        pb.set_message("work session complete".to_string());
        pb.finish_and_clear();
        //break session
        now = Local::now();
        let break_end = now + Duration::minutes(break_time.into());

        pb = ProgressBar::new(break_time as u64);
        //make the progress bar smaller
        pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{bar:40.blue/white} {msg}").expect("error"));
        //pb.set_message(format!("{} minutes remaining", break_time));
        //figure out number of seconds in one increment (divide seconds by number of increments (40))
        increment = (break_time * 60) as f64 / 40.0;
        for j in 0..(break_time * 60) {
            let remaining = break_end - Local::now();
            //minutes and seconds
            if j % increment as i32 == 0 {
                pb.inc(1);
            }
            pb.set_message(format!("{} minutes {} seconds remaining... break session {} of {}", remaining.num_minutes(), remaining.num_seconds() % 60, (i+1), iterations));
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        pb.set_message("break session complete".to_string());
        pb.finish_and_clear();
        //clear the screen
        print!("{}[2J", 27 as char);
    }
    

}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
}

fn add_event(events: &mut Vec<Event>, arg1: String, arg2: String, arg3: String, arg4: String, next_id: i32){
    //if any arguments are empty, throw error
    if arg1 == "" || arg2 == "" || arg3 == "" {
        println!("invalid usage of eadd. use --help to see usage");
        return;
    }
    let event_desc = arg1;
    let mut start_time = arg2;
    let mut end_time = arg3;
    let mut repeat = arg4;
    //remove \n from the end of due_date
    start_time = start_time.replace("\n", "");
    end_time = end_time.replace("\n", "");
    repeat = repeat.replace("\n", "");
    println!("start time: {}", start_time);
    println!("end time: {}", end_time);
    println!("repeat: {}", repeat);
    //start and end time should be in the format H:MMam || H:MMpm etc.
    //repeat should be in the format day1,day2,day3,day4,day5,day6,day7 where dayi is a day of the week
    //create a new event and add it to the event list
    //check to make sure the above criteria are met else through error
    if start_time.find(":").is_none() || (start_time.find("am").is_none() && start_time.find("pm").is_none()) {
        println!("invalid start time format. use H:MMam or H:MMpm");
        return;
    }
    if end_time.find(":").is_none() || (end_time.find("am").is_none() && end_time.find("pm").is_none()) {
        println!("invalid end time format. use H:MMam or H:MMpm");
        return;
    }
    let repeatclone = repeat.clone();
    let vecrepeat : Vec<&str> = repeatclone.split(",").collect();
    //make sure all vect elements are days of the week
    if repeat == "everyday" {
        repeat = "monday,tuesday,wednesday,thursday,friday,saturday,sunday".to_string();
    }
    if repeat == "weekday" {
        repeat = "monday,tuesday,wednesday,thursday,friday".to_string();
    }
    if repeat == "weekend" {
        repeat = "saturday,sunday".to_string();
    }
    if repeat != "" {
        for day in vecrepeat {
            if day != "sunday" && day != "monday" && day != "tuesday" && day != "wednesday" && day != "thursday" && day != "friday" && day != "saturday" && (repeat.matches("-").count() != 2) && day != "everyday" && day != "weekday" && day != "weekend" {
                println!("invalid repeat format. use subset of [monday,tuesday,wednesday,thursday,friday,saturday,sunday] separated by commas, YYYY-MM-DD, or one of [everyday,weekday,weekend]");
                return;
            }
        }
    }   
    else {
        //if repeat is empty, set repeat to date in YYYY-MM-DD format
        repeat = Local::now().format("%Y-%m-%d").to_string();
    }
    //if all criteria are met, add the event to the event list
    let event = Event {
        description: event_desc,
        start: start_time,
        end: end_time,
        repeat: repeat,
        id: next_id,
    };
    events.push(event);
    println!("event added with id {}", next_id);
}

fn daily_agenda(events: &mut Vec<Event>) {
    //get the current date
    let now = Local::now();
    let today_date = now.format("%Y-%m-%d").to_string();
    //get the day of the week as a string (monday, tuesday, etc.)
    //let today_day = now.format("%A").to_string();
    let today_day = "wednesday";
    //lowercase the day of the week
    let today_day = today_day.to_lowercase();
    let mut todays_events : Vec<Event> = Vec::new();
    //get all events that repeat on today's date or today's day of the week
    for event in events {
        if event.repeat == today_date || event.repeat.contains(&today_day) {
            let eventc = Event {
                description: event.description.clone(),
                start: event.start.clone(),
                end: event.end.clone(),
                repeat: event.repeat.clone(),
                id: event.id,
            };
            todays_events.push(eventc);
        }
    }
    todays_events.sort_by(|e1, e2| {
        let e1_vec : Vec<&str> = e1.start.split(":").collect();
        let e2_vec : Vec<&str> = e2.start.split(":").collect();
        let mut e1_hour = e1_vec[0].to_string().parse::<i32>().unwrap();
        let mut e2_hour = e2_vec[0].to_string().parse::<i32>().unwrap();
        //last two characters of e1_vec[1] are am or pm
        let e1_ampm = &e1_vec[1][e1_vec[1].len()-2..];
        let e2_ampm = &e2_vec[1][e2_vec[1].len()-2..];
        //remove am or pm from e1_vec[1] and parse to int
        let e1_min = e1_vec[1][0..e1_vec[1].len()-2].to_string().parse::<i32>().unwrap();
        let e2_min = e2_vec[1][0..e2_vec[1].len()-2].to_string().parse::<i32>().unwrap();
        if e1_hour == 12 && e1_ampm == "am" {
            e1_hour = 0;
        }
        if e2_hour == 12 && e2_ampm == "am" {
            e2_hour = 0;
        }
        if e1_hour == 12 && e1_ampm == "pm" {
            e1_hour = 0;
        }
        if e2_hour == 12 && e2_ampm == "pm" {
            e2_hour = 0;
        }
        if e1_ampm == "am" && e2_ampm == "pm" {
            return std::cmp::Ordering::Less;
        }
        else if e1_ampm == "pm" && e2_ampm == "am" {
            return std::cmp::Ordering::Greater;
        }
        else if (e1_ampm == "am" && e2_ampm == "am") || (e1_ampm == "pm" && e2_ampm == "pm") {
            if e1_hour < e2_hour {
                return std::cmp::Ordering::Less;
            }
            else if e1_hour > e2_hour {
                return std::cmp::Ordering::Greater;
            }
            else {
                if e1_min < e2_min {
                    return std::cmp::Ordering::Less;
                }
                else if e1_min > e2_min {
                    return std::cmp::Ordering::Greater;
                }
                else {
                    return std::cmp::Ordering::Equal;
                }
            }
        }
        else {
            return std::cmp::Ordering::Equal;
        }
    });
    //display the events
    println!("Today's Agenda");
    println!("---------------------------------");
    if todays_events.len() == 0 {
        println!("No events today.");
    }
    for event in todays_events {
        //pad the start and end time with spaces to be len 7
        let mut start_time = event.start;
        while start_time.len() < 7 {
            start_time.push(' ');
        }
        let mut end_time = event.end;
        while end_time.len() < 7 {
            end_time.push(' ');
        }
        println!("{} - {} - {}", start_time, end_time, event.description);
    }
    
}

fn list_event_ids(events: &mut Vec<Event>) {
    for event in events {
        println!("{} - {}", event.description, event.id);
    }

}

fn remove_event(events: &mut Vec<Event>, arg1: String) {
    //parse the event id from the arg1 string
    let event_id = arg1.parse::<i32>().unwrap();
    //find the event with the given id and remove it from the event list
    let mut index = 0;
    for event in &mut events.iter() {
        if event.id == event_id {
            events.swap_remove(index);
            println!("event {} done", event_id);
            return;
        }
        index += 1;
    }
    println!("event with id {} not found", event_id);
}

fn vayu_ui(tasks: &mut Vec<Task>, events: &mut Vec<Event>) -> io::Result<()> {
    //ratatui ui with task list, calendar, and quote of the day
    //layout
    //                      *vayu*                              
    //                  quote of the day                        
    //      task list                          weekly calendar
    let task_clone : &mut Vec<Task> = tasks;
    let event_clone : &mut Vec<Event> = events;
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui(f, task_clone, event_clone))?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let UIEvent::Key(key) = event::read()? {
                //if key is q, quit
                if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    should_quit = true;
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn ui(frame: &mut Frame, tasks: &mut Vec<Task>, events: &mut Vec<Event>) {
    //main window
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Percentage(40),
        ]
    ).split(frame.size());

    let agenda_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(4),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
        ]
    ).split(main_layout[2]);

    //one box for each day of the week starting with today as the second box
    //get the current date
    let now = Local::now();
    let today_date = now.format("%Y-%m-%d").to_string();
    //render a box with each date and day of the week starting at yesterday
    let mut day = now - Duration::days(1);
    for i in 0..7 {
        let day_date = day.format("%Y-%m-%d").to_string();
        let day_day = day.format("%A").to_string();
        let day_day = day_day.to_lowercase();
        //make a string with date + day
        let mut day_str = day_date.clone();
        let mut cat_day = day_day.clone();
        cat_day.truncate(3);
        day_str.push_str(" ");
        day_str.push_str(&cat_day);
        let day_box = Block::default().title(day_str.clone());
        //rendering the calendar
        let mut todays_events : Vec<Event> = Vec::new();
        for event in &mut *events {
            if event.repeat == day_date || event.repeat.contains(&day_day) {
                let eventc = Event {
                    description: event.description.clone(),
                    start: event.start.clone(),
                    end: event.end.clone(),
                    repeat: event.repeat.clone(),
                    id: event.id,
                };
                todays_events.push(eventc);
            }
        }
        todays_events.sort_by(|e1, e2| {
            let e1_vec : Vec<&str> = e1.start.split(":").collect();
            let e2_vec : Vec<&str> = e2.start.split(":").collect();
            let mut e1_hour = e1_vec[0].to_string().parse::<i32>().unwrap();
            let mut e2_hour = e2_vec[0].to_string().parse::<i32>().unwrap();
            //last two characters of e1_vec[1] are am or pm
            let e1_ampm = &e1_vec[1][e1_vec[1].len()-2..];
            let e2_ampm = &e2_vec[1][e2_vec[1].len()-2..];
            //remove am or pm from e1_vec[1] and parse to int
            let e1_min = e1_vec[1][0..e1_vec[1].len()-2].to_string().parse::<i32>().unwrap();
            let e2_min = e2_vec[1][0..e2_vec[1].len()-2].to_string().parse::<i32>().unwrap();
            if e1_hour == 12 && e1_ampm == "am" {
                e1_hour = 0;
            }
            if e2_hour == 12 && e2_ampm == "am" {
                e2_hour = 0;
            }
            if e1_hour == 12 && e1_ampm == "pm" {
                e1_hour = 0;
            }
            if e2_hour == 12 && e2_ampm == "pm" {
                e2_hour = 0;
            }
            if e1_ampm == "am" && e2_ampm == "pm" {
                return std::cmp::Ordering::Less;
            }
            else if e1_ampm == "pm" && e2_ampm == "am" {
                return std::cmp::Ordering::Greater;
            }
            else if (e1_ampm == "am" && e2_ampm == "am") || (e1_ampm == "pm" && e2_ampm == "pm") {
                if e1_hour < e2_hour {
                    return std::cmp::Ordering::Less;
                }
                else if e1_hour > e2_hour {
                    return std::cmp::Ordering::Greater;
                }
                else {
                    if e1_min < e2_min {
                        return std::cmp::Ordering::Less;
                    }
                    else if e1_min > e2_min {
                        return std::cmp::Ordering::Greater;
                    }
                    else {
                        return std::cmp::Ordering::Equal;
                    }
                }
            }
            else {
                return std::cmp::Ordering::Equal;
            }
        });
        //make a table with start time + description of events in todays_events and place it in the box
        let rows = todays_events.iter().map(|event| Row::new(vec![
            event.start.clone(),
            event.description.clone(),
        ]));
        let widths = [Constraint::Length(7), Constraint::Length(20)];
        let mut table = Table::new(rows, widths)
            .block(day_box)
            //.header(Row::new(vec!["Start", "Description"]).bottom_margin(1).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">>");
        if day_date == today_date {
            table = table.style(Style::default().fg(Color::Green).bg(Color::Black));
        }
        frame.render_widget(table, agenda_layout[i+1]);
        day = day + Duration::days(1);
    }
    let block = Block::default().style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::Black));
    frame.render_widget(block, agenda_layout[0]);

    let taskevents_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(4),
            Constraint::Percentage(48),
            Constraint::Percentage(48),
        ]
    ).split(main_layout[1]);

    //border on top and bottom
    frame.render_widget(
        Block::new().title("vayu dashboard - press 'q' to quit").title_alignment(Alignment::Center).style(Style::default().fg(Color::Blue).bg(Color::Black)),
        main_layout[0],
    );

    let mut table_state = TableState::default();
    
    let block_padding = Block::default().style(Style::default().fg(Color::White).bg(Color::Black));
    frame.render_widget(block_padding, taskevents_layout[0]);

    //rendering the task list
    let rows = tasks.iter().map(|task| Row::new(vec![
        task.id.to_string(),
        task.due.clone(),
        task.description.clone(),
    ]));
    let widths = [Constraint::Length(4), Constraint::Length(10), Constraint::Length(20)];
    let table = Table::new(rows, widths)
        .block(Block::default().title("Task List"))
        .header(Row::new(vec!["  ", "  ", "  "]).bottom_margin(1).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>");
    frame.render_stateful_widget(table, taskevents_layout[1], &mut table_state);

    //rendering the event list
    let rows = events.iter().map(|event| Row::new(vec![
        event.id.to_string(),
        event.start.clone(),
        event.end.clone(),
        event.description.clone(),
        event.repeat.clone().replace("monday,tuesday,wednesday,thursday,friday,saturday,sunday", "everyday").replace("monday,tuesday,wednesday,thursday,friday", "weekday").replace("saturday,sunday", "weekend"),
    ]));
    let widths = [Constraint::Length(4), Constraint::Length(10), Constraint::Length(10), Constraint::Length(20), Constraint::Length(20)];
    let table = Table::new(rows, widths)
        .block(Block::default().title("Event List"))
        .header(Row::new(vec!["  ", "  ", "  ", "  "," "]).bottom_margin(1).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>");
    frame.render_stateful_widget(table, taskevents_layout[2], &mut table_state);
    

}
