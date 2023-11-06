use crate::api::action::Action;
use crate::core::configuration::State::{
    BACKOFF, EXITED, FATAL, RUNNING, STARTING, STOPPED, STOPPING, UNKNOWN,
};
use crate::core::configuration::{AutoRestart, Configuration};
use crate::core::logger::Logger;
use crate::core::task::Task;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

pub struct Monitor {
    tasks: Arc<Mutex<BTreeMap<String, Vec<Task>>>>,
    logger: Logger,
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            tasks: Arc::new(Mutex::new(BTreeMap::new())),
            logger: Logger::new(Some("Monitor")),
        }
    }

    pub fn update_configuration(&mut self, configs: BTreeMap<String, Configuration>) -> String {
        let mut tasks = self.tasks.lock().unwrap();
        let mut result = String::new();
        self.logger.log("Configuration loading has been initiated");
        for (task_name, config) in &configs {
            let update = Task::new(&config);
            match tasks.entry(task_name.clone()) {
                Entry::Vacant(entry) => {
                    self.logger
                        .log(format!("New task: {task_name} has been added"));
                    entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                    result += &format!("{task_name}: added\n");
                }
                Entry::Occupied(mut entry) => {
                    if entry.get()[0].configuration != update.configuration {
                        entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                        self.logger.log(format!(
                            "Existing task: {task_name} was modified, changes has been applied"
                        ));
                        result += &format!("{task_name}: updated\n")
                    } else {
                        self.logger
                            .log(format!("Existing task: {task_name} wasn't modified"));
                    }
                }
            }
        }
        tasks.retain(|task_name, _task| {
            let is_present = configs.contains_key(task_name);
            if !is_present {
                result += &format!("{task_name}: deleted\n");
                self.logger.log(format!("{task_name} has been deleted"));
            }
            is_present
        });
        result
    }

    fn get_task_status(&self, task_name: Option<String>) -> String {
        let tasks = self.tasks.lock().unwrap();
        return match task_name {
            None => {
                self.logger
                    .log("Task status: no task name was specified. Returning all tasks status");
                if tasks.is_empty() {
                    return "No task found.".to_string();
                }
                tasks
                    .iter()
                    .map(|(name, task)| {
                        let process_lines: Vec<String> = task
                            .iter()
                            .enumerate()
                            .map(|(position, process)| format!("\n\t{}. {process}", position + 1))
                            .collect();
                        format!(
                            "{name}:\t\t{}\t",
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
                    self.logger
                        .log(format!("Task status: {task_name} wasn't found"));
                    format!("Can't find \"{task_name}\" task")
                }
                Some(task) => {
                    self.logger
                        .log(format!("Task status: {task_name} returning status"));
                    format!(
                        "{task_name}: {}",
                        task.iter()
                            .enumerate()
                            .map(|(position, process)| format!("\n\t{position}. {process}"))
                            .collect::<Vec<String>>()
                            .join("")
                    )
                }
            },
        };
    }

    fn get_task_json_config_by_name(&self, name: &String) -> Option<String> {
        let tasks = self.tasks.lock().unwrap();
        return match tasks
            .get(name.as_str())
            .map(|task| task[0].get_json_configuration())
        {
            None => {
                self.logger.log(format!("Get config: {name} wasn't found"));
                None
            }
            Some(config) => {
                self.logger.log(format!(
                    "Get config: returning {name} configuration in json format"
                ));
                Some(config)
            }
        };
    }

    //if it was stopped manually do need to relaunch? check conditions
    fn stop_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        self.logger.log(format!("Stop task: stopping {name}..."));
        if let Some(task) = tasks.get_mut(name) {
            for (i, process) in task.iter_mut().enumerate() {
                if let Err(e_msg) = process.stop() {
                    self.logger
                        .log(format!("Stop task: can't stop {name} #{}: {e_msg}", i + 1));
                    return Err(format!("Can't stop {name} #{i}"));
                }
            }
            Ok(())
        } else {
            self.logger.log(format!("Stop task: {name} wasn't found"));
            Err(format!("Can't find \"{name}\" task"))
        }
    }

    fn start_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        self.logger.log(format!("Start task: starting {name}..."));
        if let Some(task) = tasks.get_mut(name) {
            for (i, process) in task.iter_mut().enumerate() {
                if let Err(e_msg) = process.run() {
                    self.logger
                        .log(format!("Start task: can't start {name}..."));
                    return Err(format!("Can't run {name} #{}: {e_msg}", i + 1));
                }
            }
            Ok(())
        } else {
            Err(format!("Can't find \"{name}\" task"))
        }
    }

    fn manage_finished_state(
        process: &mut Task,
        task_name: String,
        exit_code: Option<i32>,
        logger: &Logger,
    ) {
        logger.log(format!("{task_name}: exited with status {:?}", exit_code));
        process.exit_code = exit_code;
        process.child = None;
        match process.state {
            STARTING(_) => {
                process.state = BACKOFF;
                logger.log(format!(
                    "{task_name}: Exited too quickly, status changed to backoff"
                ));
                if process.restarts_left == 0 {
                    process.state = FATAL(format!("exited too quickly"));
                    logger.log(format!(
                        "{task_name}: No restarts left, status has been changed to fatal."
                    ))
                } else {
                    process.restarts_left -= 1;
                    logger.log(format!("{task_name}: Restarting, exited too quickly"));
                    //TODO: real supervisor doesn't change status to starting if it's backoff
                    let _ = process.run();
                }
            }
            RUNNING(_) => {
                match process.configuration.auto_restart {
                    AutoRestart::True => {
                        let _ = process.run();
                        logger.log(format!("{task_name}: Relaunching..."))
                    }
                    AutoRestart::False => {
                        logger.log(format!("{task_name}: auto restart disabled."));
                        process.state = EXITED(SystemTime::now());
                    }
                    AutoRestart::Unexpected => match exit_code {
                        None => {
                            logger.log(format!("Error! Unable to access {task_name} process exit code. Can't compare with unexpected codes list"));
                            process.state = UNKNOWN;
                        }
                        Some(code) => {
                            if process.configuration.exit_codes.contains(&code) {
                                logger.log(format!("{task_name}: program has been finished with expected status, relaunch is not needed"));
                                process.state = EXITED(SystemTime::now());
                            } else {
                                logger.log(format!("{task_name}: {code} is not expected exit status, relaunching..."));
                                let _ = process.run();
                            }
                        }
                    },
                }
            }
            STOPPING(stopped_at) => {
                logger.log(format!(
                    "{task_name}: has stopped by itself after sending a signal"
                ));
                process.state = STOPPED(Some(stopped_at));
            }
            _ => logger.log_err(format!(
                "{} died in unexpected state: {}",
                task_name, process.state
            )),
        }
    }

    pub fn track(&self) {
        let monitor_clone = self.tasks.clone();
        let logger = Logger::new(Some("Monitor thread"));

        let _handle = thread::spawn(move || {
            logger.log("Monitor thread has been created");
            loop {
                let mut tasks = monitor_clone.lock().unwrap();
                for (name, task) in tasks.iter_mut() {
                    for (i, process) in task.iter_mut().enumerate() {
                        if let STARTING(started_at) = process.state {
                            if process.is_passed_starting_period(started_at) {
                                logger.log(format!("{name} #{}: is running now", i + 1));
                                process.state = RUNNING(started_at);
                            }
                        }
                        if let STOPPING(stopped_at) = process.state {
                            if process.is_passed_stopping_period(stopped_at) {
                                logger.log(format!("{name} #{}: Should be killed", i + 1));
                                //TODO: handle
                                let _ = process.kill();
                            }
                        }
                        match &mut process.child {
                            Some(child) => match child.try_wait() {
                                Ok(Some(status)) => {
                                    Monitor::manage_finished_state(
                                        process,
                                        format!("{name} #{}", i + 1),
                                        status.code(),
                                        &logger,
                                    );
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    logger.log_err(format!("Error attempting to wait: {:?}", e))
                                }
                            },
                            None => {
                                if process.configuration.auto_start
                                    && process.state == STOPPED(None)
                                {
                                    logger.log(format!("Auto starting {name} #{}", i + 1));
                                    let _ = process.run(); // TODO: handle error
                                }
                            }
                        }
                    }
                }
                drop(tasks);
                thread::sleep(Duration::from_millis(333));
            }
        });
    }

    pub fn answer(&mut self, action: Action) -> String {
        match action {
            Action::Status(status) => self.get_task_status(status),
            Action::Config(task_name) => match self.get_task_json_config_by_name(&task_name) {
                None => format!("Can't find \"{task_name}\" task"),
                Some(task) => format!("{task_name}: {task}"),
            },
            Action::Shutdown => exit(0),
            Action::Start(task_name) => match self.start_task_by_name(&task_name) {
                Ok(_) => String::new(),
                Err(err_msg) => err_msg,
            },
            Action::Stop(task_name) => match self.stop_task_by_name(&task_name) {
                Ok(_) => String::new(),
                Err(err_msg) => err_msg,
            },
            Action::Update(config_path) => {
                match Configuration::from_yml(String::from(config_path)) {
                    Ok(conf) => self.update_configuration(conf),
                    Err(err_msg) => err_msg,
                }
            }
        }
    }
}
