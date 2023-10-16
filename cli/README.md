# Time Bandit

This is a command line time management app that allows you to create `tasks` and work on task `events`. 

This can be installed from crates.io `cargo install time_bandit`

You can then view how much time you've spent total on each task,
as well as view all of the events associated with each task,
including the duration of each event as well as when the event occurred. 

The first time you run the app, an sqlite database will be created at `~/.time_bandit.db3`

Example usage: 

`$ tb task start <name of task> -details 'optional description of the task'`

This will start a new task *or* pickup where an old task has left off.

If it is a new task, the details will be a description of the task itself,
for subsequent events `-details` will add notes to the individual events.

`$ tb task list`

This will list all your tasks along with how much time you have spent on each.

`$ tb events <optional task name>`

This will list all your events along with their associated task, time stamp, duration of the event, and any details about the event.


