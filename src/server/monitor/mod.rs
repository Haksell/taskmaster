use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::core::configuration::Configuration;
use crate::core::configuration::State::FINISHED;
use crate::core::task::Task;

pub struct Monitor {
    tasks: Arc<Mutex<BTreeMap<String, Vec<Task>>>>,
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            tasks: Arc::new(Mutex::new(BTreeMap::new()))
        }
    }

    pub fn load_configuration(&mut self, configs: BTreeMap<String, Configuration>) {
        let mut tasks = self.tasks.lock().unwrap();
        for (task_name, config) in configs {
            let update = Task::new(&config);
            match tasks.entry(task_name) {
                Entry::Vacant(entry) => {
                    entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                }
                Entry::Occupied(mut entry) => {
                    if entry.get()[0].configuration != update.configuration {
                        entry.insert((0..config.num_procs).map(|_| Task::new(&config)).collect());
                    }
                }
            }
        }
        //RELAUNCH??
    }

    pub fn get_task_status(&self, task_name: Option<String>) -> String {
        let tasks = self.tasks.lock().unwrap();
        return match task_name {
            None => {
                tasks
                    .iter()
                    .map(|(name, task)| {
                        let process_lines: Vec<String> = task
                            .iter()
                            .enumerate()
                            .map(|(position, process)| format!("\n\t{}. {}", position, process))
                            .collect();
                        format!(
                            "{}: {}",
                            name,
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
                None => format!("Can't find \"{}\" task", task_name),
                Some(task) => {
                    task.iter()
                        .enumerate()
                        .map(|(position, process)| format!("\n\t{}. {}", position, process))
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            },
        };
    }

    pub fn run_autostart(&mut self) {
        let mut tasks = self.tasks.lock().unwrap();
        for (_k, task) in tasks.iter_mut() {
            for process in task {
                if process.configuration.auto_start {
                    let _ = process.run();
                }
            }
        }
    }

    pub fn get_task_json_config_by_name(&self, name: &String) -> Option<String> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(name.as_str()).map(|task| task[0].get_json_configuration())
    }
    
    //if it was stopped manually do need to relaunch? check conditions
    pub fn stop_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(name) {
            for (i, process) in task.iter_mut().enumerate() {
                if let Err(e_msg) = process.stop() {
                    return Err(format!("Can't stop {} #{}: {}", name, i, e_msg))
                }
            }
            Ok(())
        } else {
            Err(format!("Can't find \"{}\" task", name))
        }
    }
    
    pub fn start_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(name) {
            for (i, process) in task.iter_mut().enumerate() {
                if let Err(e_msg) = process.run() {
                    return Err(format!("Can't run {} #{}: {}", name, i, e_msg))
                }
            }
            Ok(())
        } else {
            Err(format!("Can't find \"{}\" task", name))
        } 
    }
    
    pub fn track(&self) {
        let monitor_clone = self.tasks.clone();
        
        let _handle = thread::spawn(move || {
            loop {
                let mut tasks = monitor_clone.lock().unwrap();
                for (name, task) in tasks.iter_mut() {
                    for (i, process) in task.iter_mut().enumerate() {
                        println!("I'm in the separate thread: [{} #{}]: {}", name, i, process.get_state());
                        if let Some(child) = &mut process.child {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    println!("{} #{} exited with status {:?}", name, i, status);
                                    process.state = FINISHED;
                                    process.exit_code = status.code();
                                    process.child = None
                                }
                                Ok(None) => {}
                                Err(e) => println!("Error attempting to wait: {:?}", e),
                            }
                        }
                    }
                }
                drop(tasks);
                thread::sleep(Duration::from_secs(1));
            }  
        });
    }
}