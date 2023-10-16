
### Config File
- **cmd**: 
  - Type: string
  - Description: The command to use to launch the program

- **numprocs**: 
  - Type: positive integer (not zero)
  - Description: The number of processes to start and keep running

- **umask**: 
  - Type: positive integer
  - Description: An umask to set before launching the program

- **workingdir**: 
  - Type: string
  - Description: A working directory to set before launching the program

- **autostart**: 
  - Type: boolean
  - Description: Whether to start this program at launch or not

- **autorestart**: 
  - Type: Autorestart enum
  - Description: Specifies if **taskmaster** should automatically restart a process if it exits when it is in the `RUNNING` state. 
  - Values: `false`, `unexpected`, or `true`

- **exitcodes**: 
  - Type: HashSet of positive integers
  - Description: Defines which return codes represent an "expected" exit status

- **startretries**: 
  - Type: positive integer
  - Description: How many times a restart should be attempted before aborting

- **starttime**: 
  - Type: positive integer
  - Description: How long the program should be running after it’s started for it to be considered "successfully started"

- **stopsignal**: 
  - Type: Signal (e.g., integer or enum)
  - Description: Specifies which signal should be used to stop (i.e., exit gracefully) the program

- **stoptime**: 
  - Type: positive integer
  - Description: How long to wait after a graceful stop before killing the program

- **stdout**: 
  - Type: string
  - Description: Options to discard the program’s standard output (stdout) or to redirect it to a file

- **stderr**: 
  - Type: string
  - Description: Options to discard the program’s standard error (stderr) or to redirect it to a file

- **env**: 
  - Type: Map of key-value pairs (String, String)
  - Description: Environment variables to set before launching the program
