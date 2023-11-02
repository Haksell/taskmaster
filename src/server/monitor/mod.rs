use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::api::action::Action;
use crate::core::configuration::Configuration;
use crate::core::configuration::State::FINISHED;
use crate::core::logger::Logger;
use crate::core::task::Task;

pub struct Monitor {
    tasks: Arc<Mutex<BTreeMap<String, Vec<Task>>>>,
    logger: Logger,
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            tasks: Arc::new(Mutex::new(BTreeMap::new())),
            logger: Logger::new(),
        }
    }

    pub fn load_configuration(&mut self, configs: BTreeMap<String, Configuration>) {
        let mut tasks = self.tasks.lock().unwrap();
        self.logger.log_with_prefix("Monitor", "Configuration loading has been initiated");
        for (task_name, config) in configs {
            let update = Task::new(&config);
            match tasks.entry(task_name.clone()) {
                Entry::Vacant(entry) => {
                    self.logger.log_with_prefix("Monitor", &format!("New task: {task_name} has been added"));
                    entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                }
                Entry::Occupied(mut entry) => {
                    if entry.get()[0].configuration != update.configuration {
                        entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                        self.logger.log_with_prefix("Monitor", &format!("Existing task: {task_name} was modified, changes has been applied"));
                        //TODO: RELAUNCH??
                    } else {
                        self.logger.log_with_prefix("Monitor", &format!("Existing task: {task_name} wasn't modified"));
                    }
                }
            }
        }
    }

    fn get_task_status(&self, task_name: Option<String>) -> String {
        let tasks = self.tasks.lock().unwrap();
        return match task_name {
            None => {
                self.logger.log_with_prefix("Monitor", "Task status: no task name was specified. Returning all tasks status");
                tasks
                    .iter()
                    .map(|(name, task)| {
                        let process_lines: Vec<String> = task
                            .iter()
                            .enumerate()
                            .map(|(position, process)| format!("\n\t{position}. {process}"))
                            .collect();
                        format!(
                            "{name}: {}",
                            if task.len() == 1 {
                                task[0].to_string()
                            } else {
                                process_lines.join("")
                            }
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Some(ref task_name) => match tasks.get(task_name.as_str()) {
                None => {
                    self.logger.log_with_prefix("Monitor", &format!("Task status: {task_name} wasn't found"));
                    format!("Can't find \"{task_name}\" task")
                }
                Some(task) => {
                    self.logger.log_with_prefix("Monitor", &format!("Task status: {task_name} returning status"));
                    format!("{task_name}: {}",
                            task.iter()
                                .enumerate()
                                .map(|(position, process)| format!("\n\t{position}. {process}"))
                                .collect::<Vec<String>>()
                                .join(""))
                }
            },
        };
    }

    pub fn run_autostart(&mut self) {
        let mut tasks = self.tasks.lock().unwrap();
        for (task_name, task) in tasks.iter_mut() {
            for (i, process) in task.iter_mut().enumerate() {
                if process.configuration.auto_start {
                    self.logger.log_with_prefix("Monitor", &format!("Autostarting {task_name} #{}", i + 1));
                    let _ = process.run();
                }
            }
        }
    }

    fn get_task_json_config_by_name(&self, name: &String) -> Option<String> {
        let tasks = self.tasks.lock().unwrap();
        return match tasks.get(name.as_str()).map(|task| task[0].get_json_configuration()) {
            None => {
                self.logger.log_with_prefix("Monitor", &format!("Get config: {name} wasn't found"));
                None
            }
            Some(config) => {
                self.logger.log_with_prefix("Monitor", &format!("Get config: returning {name} configuration in json format"));
                Some(config)
            }
        };
    }

    //if it was stopped manually do need to relaunch? check conditions
    fn stop_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        self.logger.log_with_prefix("Monitor", &format!("Stop task: stopping {name}..."));
        if let Some(task) = tasks.get_mut(name) {
            for (i, process) in task.iter_mut().enumerate() {
                if let Err(e_msg) = process.stop() {
                    self.logger.log_with_prefix("Monitor", &format!("Stop task: can't stop {name} #{}: {e_msg}", i + 1));
                    return Err(format!("Can't stop {name} #{i}"));
                }
            }
            Ok(())
        } else {
            self.logger.log_with_prefix("Monitor", &format!("Stop task: {name} wasn't found"));
            Err(format!("Can't find \"{name}\" task"))
        }
    }

    fn start_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        self.logger.log_with_prefix("Monitor", &format!("Start task: starting {name}..."));
        if let Some(task) = tasks.get_mut(name) {
            for (i, process) in task.iter_mut().enumerate() {
                if let Err(e_msg) = process.run() {
                    self.logger.log_with_prefix("Monitor", &format!("Start task: can't start {name}..."));
                    return Err(format!("Can't run {name} #{}: {e_msg}", i + 1));
                }
            }
            Ok(())
        } else {
            Err(format!("Can't find \"{name}\" task"))
        }
    }

    pub fn track(&self) {
        let monitor_clone = self.tasks.clone();
        let logger = Logger::new();
        logger.log("Starting track the tasks...");

        let _handle = thread::spawn(move || {
            logger.log_with_prefix("Monitor thread", "Monitor thread has been created");
            loop {
                let mut tasks = monitor_clone.lock().unwrap();
                for (name, task) in tasks.iter_mut() {
                    for (i, process) in task.iter_mut().enumerate() {
                        if let Some(child) = &mut process.child {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    logger.log_with_prefix("Monitor thread", &format!("{name} #{} exited with status {:?}", i + 1, status));
                                    process.state = FINISHED;
                                    process.exit_code = status.code();
                                    process.child = None
                                }
                                Ok(None) => {}
                                Err(e) => logger.log_err(format!("Error attempting to wait: {:?}", e)),
                            }
                        }
                    }
                }
                drop(tasks);
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    pub fn answer(&mut self, action: Action) -> String {
        match action {
            Action::Status(status) => self.get_task_status(status),
            Action::Help => action.get_description(),
            Action::Config(task_name) => {
                match self.get_task_json_config_by_name(&task_name) {
                    None => format!("Can't find \"{task_name}\" task"),
                    Some(task) => format!("{task_name}: {task}"),
                }
            }
            Action::Exit => exit(0),
            Action::Start(task_name) => match self.start_task_by_name(&task_name) {
                Ok(_) => String::new(),
                Err(err_msg) => err_msg,
            },
            Action::Stop(task_name) => match self.stop_task_by_name(&task_name) {
                Ok(_) => String::new(),
                Err(err_msg) => err_msg,
            },
        }
    }
}