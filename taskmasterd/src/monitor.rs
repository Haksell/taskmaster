use crate::action::Action;
use crate::configuration::State::{
    BACKOFF, EXITED, FATAL, RUNNING, STARTING, STOPPED, STOPPING, UNKNOWN,
};
use crate::configuration::{AutoRestart, Configuration};
use crate::logger::Logger;
use crate::remove_and_exit;
use crate::task::Task;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, SystemTime};

pub struct Monitor {
    tasks: Arc<Mutex<BTreeMap<String, Vec<Task>>>>,
    logger: Arc<Mutex<Logger>>,
    config_path: String,
}

impl Monitor {
    pub fn new(config_path: String, logger: Arc<Mutex<Logger>>) -> Monitor {
        Monitor {
            tasks: Arc::new(Mutex::new(BTreeMap::new())),
            logger,
            config_path,
        }
    }

    pub fn update_configuration(&mut self, configs: BTreeMap<String, Configuration>) -> String {
        let mut tasks = self.tasks.lock().unwrap();
        let mut logger = self.logger.lock().unwrap();
        let mut result = String::new();
        logger.monit_log("Configuration loading has been initiated".to_string());
        for (task_name, config) in &configs {
            let update = Task::new(&config);
            match tasks.entry(task_name.clone()) {
                Entry::Vacant(entry) => {
                    logger.monit_log(format!("New task: {task_name} has been added"));
                    entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                    result += &format!("{task_name}: added\n");
                }
                Entry::Occupied(mut entry) => {
                    if entry.get()[0].configuration != update.configuration {
                        entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                        logger.monit_log(format!(
                            "Existing task: {task_name} was modified, changes has been applied"
                        ));
                        result += &format!("{task_name}: updated\n")
                    } else {
                        logger.monit_log(format!("Existing task: {task_name} wasn't modified"));
                    }
                }
            }
        }
        tasks.retain(|task_name, _task| {
            let is_present = configs.contains_key(task_name);
            if !is_present {
                result += &format!("{task_name}: deleted\n");
                logger.monit_log(format!("{task_name} has been deleted"));
            }
            is_present
        });
        result
    }

    fn get_task_status(&mut self, task_name: Option<String>) -> String {
        let tasks = self.tasks.lock().unwrap();
        let mut logger = self.logger.lock().unwrap();
        return match task_name {
            None => {
                logger.monit_log(
                    "Task status: no task name was specified. Returning all tasks status"
                        .to_string(),
                );
                if tasks.is_empty() {
                    return "No task found.".to_string();
                }
                tasks
                    .iter()
                    .map(|(name, task)| {
                        let process_lines: Vec<String> = task
                            .iter()
                            .enumerate()
                            .map(|(position, process)| format!("\n\t{position}. {process}"))
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
                    logger.monit_log(format!("Task status: {task_name} wasn't found"));
                    format!("Can't find \"{task_name}\" task")
                }
                Some(task) => {
                    logger.monit_log(format!("Task status: {task_name} returning status"));
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

    fn get_task_json_config_by_name(&mut self, name: &String) -> Option<String> {
        let tasks = self.tasks.lock().unwrap();
        let mut logger = self.logger.lock().unwrap();
        return match tasks
            .get(name.as_str())
            .map(|task| task[0].get_json_configuration())
        {
            None => {
                logger.monit_log(format!("Get config: {name} wasn't found"));
                None
            }
            Some(config) => {
                logger.monit_log(format!(
                    "Get config: returning {name} configuration in json format"
                ));
                Some(config)
            }
        };
    }

    fn stop_task(&mut self, name: &String, num: &Option<usize>) -> String {
        let mut tasks = self.tasks.lock().unwrap();
        let mut logger = self.logger.lock().unwrap();
        let mut result = String::new();
        if let Some(task_group) = tasks.get_mut(name) {
            match num {
                None => {
                    logger.monit_log(format!("All task in {name} will be stopped"));
                    for (i, process) in task_group.iter_mut().enumerate() {
                        if let RUNNING(_) = process.state {
                            if let Err(e_msg) = process.stop() {
                                result += &logger.monit_log(format!(
                                    "{name}#{i}: Error during the stop: {e_msg}\n"
                                ));
                            } else {
                                result += &logger.monit_log(format!("{name}#{i}: Stopping...\n"))
                            }
                        } else {
                            result += &logger.monit_log(format!(
                                "{name}#{i}: Can't be stopped. Current status {}\n",
                                process.state
                            ))
                        }
                    }
                }
                Some(index) => match task_group.get_mut(*index) {
                    None => {
                        result += &logger.monit_log(format!(
                            "{name}#{index}: Can't be stopped, it doesn't exist\n"
                        ))
                    }
                    Some(task) => match task.stop() {
                        Ok(_) => {
                            result += &logger.monit_log(format!("{name}#{index}: Stopping...\n"))
                        }
                        Err(err) => {
                            result += &logger
                                .monit_log(format!("{name}#{index}: Can't be stopped: {err}\n"))
                        }
                    },
                },
            }
        } else {
            return format!("Can't find \"{name}\" task");
        }
        result
    }

    fn start_task(&mut self, name: &String, num: &Option<usize>) -> String {
        let mut tasks = self.tasks.lock().unwrap();
        let mut logger = self.logger.lock().unwrap();
        let mut result = String::new();
        if let Some(task_group) = tasks.get_mut(name) {
            match num {
                None => {
                    logger.monit_log(format!("All task in {name} will be started"));
                    for (i, process) in task_group.iter_mut().enumerate() {
                        if process.can_be_launched() {
                            if let Err(e_msg) = process.run() {
                                result += &logger.monit_log(format!(
                                    "{name}#{i}: Error during the start: {e_msg}\n"
                                ));
                            } else {
                                result += &logger.monit_log(format!("{name}#{i}: Starting...\n"))
                            }
                        } else {
                            result += &logger.monit_log(format!(
                                "{name}#{i}: Can't be started. Current status {}\n",
                                process.state
                            ))
                        }
                    }
                }
                Some(index) => match task_group.get_mut(*index) {
                    None => {
                        result += &logger.monit_log(format!(
                            "{name}#{index}: Can't be started, it doesn't exist\n"
                        ))
                    }
                    Some(task) => match task.run() {
                        Ok(_) => {
                            result +=
                                &logger.monit_log(format!("{name}#{index}: has been started\n"))
                        }
                        Err(err) => {
                            result += &logger
                                .monit_log(format!("{name}#{index}: Can't be launched: {err}\n"))
                        }
                    },
                },
            }
        } else {
            return format!("Can't find \"{name}\" task");
        }
        result
    }

    fn manage_finished_state(
        process: &mut Task,
        task_name: String,
        exit_code: Option<i32>,
        logger: &mut MutexGuard<Logger>,
    ) {
        logger.sth_log(format!("{task_name}: exited with status {:?}", exit_code));
        process.exit_code = exit_code;
        process.child = None;
        match process.state {
            STARTING(_) => {
                process.state = BACKOFF;
                logger.sth_log(format!(
                    "{task_name}: Exited too quickly, status changed to backoff"
                ));
                if process.restarts_left == 0 {
                    process.state = FATAL(format!("exited too quickly"));
                    logger.sth_log(format!(
                        "{task_name}: No restarts left, status has been changed to fatal."
                    ));
                } else {
                    process.restarts_left -= 1;
                    logger.sth_log(format!("{task_name}: Restarting, exited too quickly"));
                    //TODO: real supervisor doesn't change status to starting if it's backoff
                    let _ = process.run();
                }
            }
            RUNNING(_) => {
                match process.configuration.auto_restart {
                    AutoRestart::True => {
                        let _ = process.run();
                        logger.sth_log(format!("{task_name}: Relaunching..."));
                    }
                    AutoRestart::False => {
                        logger.sth_log(format!("{task_name}: auto restart disabled."));
                        process.state = EXITED(SystemTime::now());
                    }
                    AutoRestart::Unexpected => match exit_code {
                        None => {
                            logger.sth_log(format!("Unable to access {task_name} process exit code. Can't compare with unexpected codes list"));
                            process.state = UNKNOWN;
                        }
                        Some(code) => {
                            if process.configuration.exit_codes.contains(&code) {
                                logger.sth_log(format!("{task_name}: program has been finished with expected status, relaunch is not needed"));
                                process.state = EXITED(SystemTime::now());
                            } else {
                                logger.sth_log(format!("{task_name}: {code} is not expected exit status, relaunching..."));
                                let _ = process.run();
                            }
                        }
                    },
                }
            }
            STOPPING(stopped_at) => {
                logger.sth_log(format!(
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
        let logger_clone = self.logger.clone();

        let _handle = thread::spawn(move || {
            loop {
                let mut tasks = monitor_clone.lock().unwrap();
                let mut logger = logger_clone.lock().unwrap();
                for (name, task) in tasks.iter_mut() {
                    for (i, process) in task.iter_mut().enumerate() {
                        if let STARTING(started_at) = process.state {
                            if process.is_passed_starting_period(started_at) {
                                logger.sth_log(format!("{name} #{i}: is running now"));
                                process.state = RUNNING(started_at);
                            }
                        }
                        if let STOPPING(stopped_at) = process.state {
                            if process.is_passed_stopping_period(stopped_at) {
                                logger.sth_log(format!("{name} #{i}: Should be killed"));
                                //TODO: handle
                                let _ = process.kill();
                            }
                        }
                        match &mut process.child {
                            Some(child) => match child.try_wait() {
                                Ok(Some(status)) => {
                                    Monitor::manage_finished_state(
                                        process,
                                        format!("{name} #{i}"),
                                        status.code(),
                                        &mut logger,
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
                                    logger.sth_log(format!("Auto starting {name} #{i}"));
                                    if let Err(error_msg) = process.run() {
                                        logger.sth_log(format!("{name}#{i}: {error_msg}"));
                                    }
                                }
                            }
                        }
                    }
                }
                drop(logger);
                drop(tasks);
                thread::sleep(Duration::from_millis(100));
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
            Action::Shutdown => remove_and_exit(0),
            Action::Start(arg) => match arg {
                Some((task_name, num)) => self.start_task(&task_name, &num),
                None => {
                    let tasks = self
                        .tasks
                        .lock()
                        .unwrap()
                        .keys()
                        .cloned()
                        .collect::<Vec<String>>();
                    tasks
                        .iter()
                        .map(|task_name| self.start_task(&task_name, &None))
                        .collect()
                }
            },
            Action::Stop(arg) => match arg {
                Some((task_name, num)) => self.stop_task(&task_name, &num),
                None => {
                    let tasks = self
                        .tasks
                        .lock()
                        .unwrap()
                        .keys()
                        .cloned()
                        .collect::<Vec<String>>();
                    tasks
                        .iter()
                        .map(|task_name| self.stop_task(&task_name, &None))
                        .collect()
                }
            },
            Action::Update(arg) => {
                if let Some(config_path) = arg {
                    self.config_path = config_path;
                }
                match Configuration::from_yml(self.config_path.clone(), self.logger.clone()) {
                    Ok(conf) => self.update_configuration(conf),
                    Err(err_msg) => format!("{err_msg}"),
                }
            }
            Action::Maintail(arg) => self
                .logger
                .lock()
                .unwrap()
                .get_history(match arg {
                    Some(num_lines) => num_lines,
                    None => 10,
                })
                .join(""),
        }
    }
}