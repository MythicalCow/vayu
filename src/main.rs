extern crate clap;
extern crate colored;
extern crate chrono;
extern crate indicatif;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use colored::Colorize;
use chrono::{Local,Duration};
use std::io::prelude::*;
use chrono::Datelike;
use indicatif::ProgressBar;



//struct for the main command.
#[derive(Parser)]
struct Command {
    /// {add,list,done,pomo} add: add a new task, list: list all tasks, done: mark a task as done, pomo: pomodoro timer ex: vayu pomo 3 50 10 (3 sessions of 50 minute work and 10 minute break)
    command: String,
    /// {add} task description and due date (YYYY-MM-DD or today,yesterday,monday,etc.) separated by a colon. Ex: "vayu add yoga due:today" {done} task id. Ex: "vayu done 1"
    #[clap(default_value = "")]
    arg1: String,
    #[clap(default_value = "")]
    arg2: String,
    #[clap(default_value = "")]
    arg3: String,
    /// {eadd} used for specifiying event repeat. ex: "vayu eadd yoga 8:00am 9:00am monday,tuesday,friday" 
    #[clap(default_value = "")]
    arg4: String,
}

//struct for a task. there are some weird warnings about this being unused
#[allow(dead_code)]
struct Task {
    description: String,
    due: String,
    done: bool,
    id: i32,
}

#[allow(dead_code)]
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
    else if command.command == "pomo" {
        pomodoro(command.arg1, command.arg2, command.arg3);
    }
    else if command.command == "eadd" {
        add_event(&mut events, command.arg1, command.arg2, command.arg3, command.arg4, next_event_id);
    }
    else if command.command == "elist" {
        daily_agenda(&mut events);
    }
    else if command.command == "eids" {
        list_event_ids(&mut events);
    }
    else if command.command == "erem" {
        remove_event(&mut events, command.arg1);
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
                println!("{}| {} | {}", id.on_red(), task.due.on_red(), task.description.on_red());
            }
            else if i % 2 == 0 {
                println!("{}| {} | {}", id.on_black(), task.due.on_black(), task.description.on_black());
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
    else if due_date == "sunday"{
        let days_until_sunday = 7 - now.weekday().num_days_from_sunday();
        due_date = (now + Duration::days(days_until_sunday.into())).format("%Y-%m-%d").to_string();
    }
    else if due_date == "monday"{
        let days_until_monday = 1 - now.weekday().num_days_from_sunday();
        due_date = (now + Duration::days(days_until_monday.into())).format("%Y-%m-%d").to_string();
    }
    else if due_date == "tuesday"{
        let days_until_tuesday = 2 - now.weekday().num_days_from_sunday();
        due_date = (now + Duration::days(days_until_tuesday.into())).format("%Y-%m-%d").to_string();
    }
    else if due_date == "wednesday"{
        let days_until_wednesday = 3 - now.weekday().num_days_from_sunday();
        due_date = (now + Duration::days(days_until_wednesday.into())).format("%Y-%m-%d").to_string();
    }
    else if due_date == "thursday"{
        let days_until_thursday = 4 - now.weekday().num_days_from_sunday();
        due_date = (now + Duration::days(days_until_thursday.into())).format("%Y-%m-%d").to_string();
    }
    else if due_date == "friday"{
        let days_until_friday = 5 - now.weekday().num_days_from_sunday();
        due_date = (now + Duration::days(days_until_friday.into())).format("%Y-%m-%d").to_string();
    }
    else if due_date == "saturday"{
        let days_until_saturday = 6 - now.weekday().num_days_from_sunday();
        due_date = (now + Duration::days(days_until_saturday.into())).format("%Y-%m-%d").to_string();
    }
    else if due_date == "tomorrow" {
        due_date = (now + Duration::days(1)).format("%Y-%m-%d").to_string();
    } 
    else if due_date == "yesterday" {
        due_date = (now - Duration::days(1)).format("%Y-%m-%d").to_string();
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

fn pomodoro(arg1: String, arg2: String, arg3: String){
    //we will use the chrono crate to get the current time and to calculate the time remaining
    //we will use indicatif to display a progress bar
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
            pb.set_message(format!("{} minutes {} seconds remaining... work session {} of {}", remaining.num_minutes(), remaining.num_seconds() % 60, (i+1).to_string().green(), iterations.to_string().green()));
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
            pb.set_message(format!("{} minutes {} seconds remaining... break session {} of {}", remaining.num_minutes(), remaining.num_seconds() % 60, (i+1).to_string().green(), iterations.to_string().green()));
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
    let vecrepeat : Vec<&str> = repeat.split(",").collect();
    //make sure all vect elements are days of the week
    if repeat != "" {
        for day in vecrepeat {
            if day != "sunday" && day != "monday" && day != "tuesday" && day != "wednesday" && day != "thursday" && day != "friday" && day != "saturday" {
                println!("invalid repeat format. use subset of [monday,tuesday,wednesday,thursday,friday,saturday,sunday] separated by commas");
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
    println!("event added with id {}", next_id.to_string().green());
}

fn daily_agenda(events: &mut Vec<Event>) {
    //get the current date
    let now = Local::now();
    let today_date = now.format("%Y-%m-%d").to_string();
    //get the day of the week as a string (monday, tuesday, etc.)
    let today_day = now.format("%A").to_string();
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
        let e1_hour = e1_vec[0].to_string().parse::<i32>().unwrap();
        let e2_hour = e2_vec[0].to_string().parse::<i32>().unwrap();
        //last two characters of e1_vec[1] are am or pm
        let e1_ampm = &e1_vec[1][e1_vec[1].len()-2..];
        let e2_ampm = &e2_vec[1][e2_vec[1].len()-2..];
        //remove am or pm from e1_vec[1] and parse to int
        let e1_min = e1_vec[1][0..e1_vec[1].len()-2].to_string().parse::<i32>().unwrap();
        let e2_min = e2_vec[1][0..e2_vec[1].len()-2].to_string().parse::<i32>().unwrap();
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
            println!("event {} done", event_id.to_string().green());
            return;
        }
        index += 1;
    }
    println!("event with id {} not found", event_id.to_string().red());
}