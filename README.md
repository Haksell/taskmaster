# Rustmaster

## Description

Project is a simple implementation of taskmaster in Rust. It is a daemon that runs and monitors other programs.
It is a simple copy of supervisor. The project is done using Vagrant virtual machine to be able to run it with sudo on
school's PC.

# Menu

- [Run](#id-section0)
- [Client Commands](#id-section2)
- [Stop](#id-section4)
- [Config file](#id-section1)

<div id='id-section0'/>

## Run

-------------

### Launch Virtual Machine

#### using Makefile

```bash
make vagrant
```

-------------

### Run server

Daemonized server should be run with sudo

```bash
sudo make daemon
```

Debug server can be run without sudo:

```bash
make nodaemon
```

![Alt text](.images/1%20-%20Debug%20mode.png "Terminal taskmaster in debug mode")


-------------

### Run client

```bash
make client
```

### List of possible client commands:

<div id='id-section2'/>

- help

![Alt text](.images/2%20-%20Client%20help%20example.png "Terminal taskmaster client help example")

- status - returns a list of programs with their statuses

![Alt text](.images/5%20-%20Client%20status.png "Terminal taskmaster client status command example")

- start - starts task by its name

![Alt text](.images/4%20-%20Client%20start.png "Terminal taskmaster client start command example")

- stop - stops task by its name

![Alt text](.images/6%20-%20Client%20stop.png "Terminal taskmaster client stop command example")

- restart - restarts task by its name

![Alt text](.images/10%20-%20Client%20restart.png "Terminal taskmaster client restart command example")

- config - returns a task configuration in json.

![Alt text](.images/7%20-%20Client%20config.png "Terminal taskmaster client config command example")

- tail can read last N lines of stdout and stderr of log files or stream them to the console

![Alt text](.images/3%20-%20Client%20tail%20command.gif "Terminal taskmaster client tail command example")
  
- shutdown - Shut the remote taskmasterd down
- exit && quit - Closes the CLI
- http - transfer logs via http
- update - Reloads the configuration tracking changes

![Alt text](.images/8%20-%20Client%20update.png "Terminal taskmaster client update command example")

- maintail - tail daemon logs
  
![Alt text](.images/9%20-%20Client%20maintail.png "Terminal taskmaster client maintail command example")

- signal - send signal to task process

![Alt text](.images/11%20-%20Client%20signal.png "Terminal taskmaster client signal command example")

-------------

<div id='id-section4'/>

### Stop server

Stop daemonized server should be run with sudo

```bash
sudo make stop
```

Debug server can be stopped without sudo:

```bash
make stop
```

-------------

### Clean

```bash
make clean
```

-------------


<div id='id-section1'/>

## Config

**Only cmd field is mandatory for config, for other they are default values**

- **cmd**:
    - Type: string
    - Default value: No default value
    - Description: The command to use to launch the program

- **num_procs**:
    - Type: positive integer (not zero)
    - Default value: 1
    - Description: The number of processes to start and keep running

- **umask**:
    - Type: positive integer
    - Default value: 0o022
    - Description: An umask to set before launching the program

- **working_dir**:
    - Type: string
    - Default value: current dir
    - Description: A working directory to set before launching the program

- **auto_start**:
    - Type: boolean
    - Default value: true
    - Description: Whether to start this program at launch or not

- **auto_restart**:
    - Type: Autorestart enum
    - Default value: unexpected
    - Description: Specifies if **taskmaster** should automatically restart a process if it exits when it is in
      the `RUNNING` state.
    - Values: `false`, `unexpected`, or `true`

- **exit_codes**:
    - Type: Vector of positive integers (**maybe change to set in the future**)
    - Default value: [0]
    - Description: Defines which return codes represent an "expected" exit status

- **start_retries**:
    - Type: positive integer
    - Default value: 3
    - Description: How many times a restart should be attempted before aborting

- **start_time**:
    - Type: positive integer
    - Default value: 1
    - Description: How long the program should be running after it’s started for it to be considered "successfully
      started"

- **stop_signal**:
    - Type: Signal (e.g., integer or enum)
    - Default value: TERM
    - Description: Specifies which signal should be used to stop (i.e., exit gracefully) the program

- **stop_time**:
    - Type: positive integer
    - Default value: 10
    - Description: How long to wait after a graceful stop before killing the program

- **stdout**:
    - Type: string
    - Default value: None (**need to decide, maybe /tmp/taskname.stdout**)
    - Description: Options to discard the program’s standard output (stdout) or to redirect it to a file

- **stderr**:
    - Type: string
    - Default value: None (**need to decide, maybe /tmp/taskname.stderr**)
    - Description: Options to discard the program’s standard error (stderr) or to redirect it to a file

- **env**:
    - Type: Map of key-value pairs (String, String)
    - Default value: Empty
    - Description: Environment variables to set before launching the program
