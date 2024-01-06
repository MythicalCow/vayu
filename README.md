![banner](banner.png)
[![Crate](https://img.shields.io/crates/v/vayu.svg)](https://crates.io/crates/vayu)

# Description
Vayu is a command-line interface like taskwarrior meant for task management. The end goal for the tool is to have a multipurpose study tool with task and note storage, task management, and pomodoro study timers/music. As a computer engineering student at UIUC, having an organized task management system is critical. I hope to bring that to others with vayu.
# Demo Images
![demo](demo.png)
![demo2](demo2.png)
# Installation
The best way to install vayu is via cargo which comes with an installation of rust. For installation of rust navigate to https://www.rust-lang.org/tools/install.

Then in Windows PowerShell or the approriate Terminal for your OS type in:
`cargo install vayu`

To check if vayu has been installed properly open a new terminal window and type:
`vayu list`

If there is an empty list then congratulations the installation was successful! If not ensure rust is included in your path variable as explained on the rust installation page.
# Usage and Examples
# main dashboard (calendar, task list, event list)
`vayu`
### to view a list of current tasks use:
`vayu list`
### to add a task use:
`vayu add "task description due:YYYY-MM-DD"`
note that YYYY-MM-DD can also be replaced with today, tomorrow, and days of the week
### to mark tasks as done:
`vayu done id`
where id is the listed id of the task viewable through `vayu list`
### to use the pomodoro timer:
`vayu pomo iterations work_session_time break_session_time`
### lists events scheduled for the day
`vayu elist` 
### adding a new event
`vayu eadd "description" start_time end_time repeat`
adds a new event that potentially repeats. `start_time` and `end_time` are provided in the format HH:MMam or HH:MMpm. 
repeat is an optional arg which if left empty is set to only schedule an event for that day. 
Otherwise repeat is provided as days of the week selected from `{monday,wednesday,tuesday,...}` separated by commas. 
shortcuts for repeat are developed but will be released on the next update. these are {everyday,weekday,weekend} 
Ex: `vayu eadd "yoga" 4:30am 5:30am tuesday,thursday` 
### list ids of all events
`vayu eids` 
### remove events
`vayu erem` 
Ex: `vayu erem 1`
# Contributions
vayu wouldn't be possible without several great rust crates. For a full list of dependencies please checkout the `Cargo.toml` file. 
