use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::core::configuration::Configuration;
use crate::core::task::Task;

pub struct Monitor {
    tasks: Arc<Mutex<BTreeMap<String, Task>>>,
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
            match tasks.get(task_name.as_str()) {
                None => {
                    tasks.insert(task_name, Task::new(config));
                }
                Some(element) => {
                    let update = Task::new(config);
                    if element.configuration != update.configuration {
                        tasks.insert(task_name, update);
                        //RESTART
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
                let mut result = String::new();
                for (i, (name, task)) in tasks.iter().enumerate() {
                    result += format!("{}: {}", name, task).as_str();
                    if i != tasks.len() - 1 {
                        result += "\n"
                    }
                }
                result
            }
            Some(task_name) => match tasks.get(task_name.as_str()) {
                None => format!("Can't find \"{}\" task", task_name),
                Some(task) => format!("{}: {}", task_name, task.to_string()),
            },
        };
    }

    pub fn run_autostart(&mut self) {
        let mut tasks = self.tasks.lock().unwrap();
        for (_k, task) in tasks.iter_mut() {
            if task.configuration.auto_start {
                let _ = task.run();
            }
        }
    }

    pub fn get_task_json_config_by_name(&self, name: &String) -> Option<String> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(name.as_str()).map(|task| task.get_json_configuration())
    }
    
    //if it was stopped manually do need to relaunch? check conditions
    pub fn stop_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(name) {
            if let Err(e_msg) = task.stop() {
                return Err(format!("Can't stop {}: {}", name, e_msg))
            }
            Ok(())
        } else {
            Err(format!("Can't find \"{}\" task", name))
        }
    }
    
    pub fn start_task_by_name(&mut self, name: &String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(name) {
            if let Err(e_msg) = task.run() {
                return Err(format!("Can't run {}: {}", name, e_msg))
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
                let tasks = monitor_clone.lock().unwrap();
                for (name, task) in tasks.iter() {
                    println!("I'm in the separate thread: [{}]: {}", name, task.get_state());
                }
                drop(tasks);
                thread::sleep(Duration::from_secs(1));
            }  
        });
        //handle.join().unwrap()
    }


    /*
    fn prototype() {
        let mut children = vec![child1, child2, child3];

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let mut input = String::new();
            println!("Enter the index of the child process to kill:");
            std::io::stdin().read_line(&mut input).unwrap();
            if let Ok(index) = input.trim().parse::<usize>() {
                tx.send(index).unwrap();
            } else {
                println!("Invalid input. Please enter a valid index.");
            }
        });

        while !children.is_empty() {
            if let Ok(index) = rx.try_recv() {
                if index < children.len() {
                    children[index].kill().unwrap();
                    println!("Killed child process at index {}", index);
                } else {
                    println!("Invalid index.");
                }
            }

            let mut terminated: Vec<usize> = vec![];
            for (i, child) in children.iter_mut().enumerate() {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        println!("{:?} exited with status {:?}", child, status);
                        terminated.push(i);
                    }
                    Ok(None) => {}
                    Err(e) => println!("Error attempting to wait: {:?}", e),
                }
            }

            children = children
                .into_iter()
                .enumerate()
                .filter(|(i, _child)| !terminated.contains(i))
                .map(|(_i, child)| child)
                .collect::<Vec<Child>>();
        }
    }
    
     */
}