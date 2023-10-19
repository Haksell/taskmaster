# Rustmaster

# My Table of content
- [Run](#id-section0)
- [Config](#id-section1)
- [Client Commands](#id-section2)

<div id='id-section0'/>

## Run

! for the moment it's server will only echo client messages from stdin

#### Launch Virtual Machine
```bash
vagrant up && vagrant ssh
```

#### Run server
```bash
rm -rf .unixdomain.sock && cargo run --bin server
```
! **rm -rf .unixdomain.sock** is here to remove created unix domain socket (will be handled after implementing signals)

#### Run client
```bash
cargo run --bin client
```

#### Run tests
```bash
cargo test
```

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
  - Description: Specifies if **taskmaster** should automatically restart a process if it exits when it is in the `RUNNING` state. 
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
  - Description: How long the program should be running after it’s started for it to be considered "successfully started"

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


<div id='id-section2'/>

## Client Commands
- **status**:
  - Description: Returns a list of programs with their statuses (start/stop/restart) and program names.

- **reload**:
  - Description: Reloads the configuration while tracking changes.

- **stop all**:
  - Description: Stops all programs.
