use std::collections::BTreeMap;
use crate::core::configuration::Configuration;
use crate::core::task::Task;

pub struct Monitor {
    tasks: BTreeMap<String, Task>,
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            tasks: BTreeMap::new()
        }
    }

    pub fn load_configuration(&mut self, configs: BTreeMap<String, Configuration>) {
        for (task_name, config) in configs {
            match self.tasks.get(task_name.as_str()) {
                None => {
                    self.tasks.insert(task_name, Task::new(config));
                }
                Some(element) => {
                    let update = Task::new(config);
                    if element.configuration != update.configuration {
                        self.tasks.insert(task_name, update);
                        //RESTART
                    }
                }
            }
        }
        //RELAUNCH??
    }

    pub fn get_task_status(&self, task_name: Option<String>) -> String {
        return match task_name {
            None => {
                let mut result = String::new();
                for (i, (name, task)) in self.tasks.iter().enumerate() {
                    result += format!("{}: {}", name, task).as_str();
                    if i != self.tasks.len() - 1 {
                        result += "\n"
                    }
                }
                result
            }
            Some(task_name) => match self.tasks.get(task_name.as_str()) {
                None => format!("Can't find \"{}\" task", task_name),
                Some(task) => format!("{}: {}", task_name, task.to_string()),
            },
        };
    }

    pub fn run_autostart(&mut self) {
        for (_k, task) in self.tasks.iter_mut() {
            if task.configuration.auto_start {
                let _ = task.run();
            }
        }
    }

    pub fn get_task_by_name(&self, name: &String) -> Option<&Task> {
        return self.tasks.get(name.as_str());
    }
    
    //if it was stopped manually do need to relaunch? check conditions
    pub fn stop_task_by_name(&mut self, name: &String) -> Result<(), String> {
        if let Some(task) = self.tasks.get_mut(name) {
            if let Err(e_msg) = task.stop() {
                return Err(format!("Can't stop {}: {}", name, e_msg))
            }
            Ok(())
        } else {
            Err(format!("Can't find \"{}\" task", name))
        }
    }
    
    pub fn start_task_by_name(&mut self, name: &String) -> Result<(), String> {
        if let Some(task) = self.tasks.get_mut(name) {
            if let Err(e_msg) = task.run() {
                return Err(format!("Can't run {}: {}", name, e_msg))
            }
            Ok(())
        } else {
            Err(format!("Can't find \"{}\" task", name))
        } 
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