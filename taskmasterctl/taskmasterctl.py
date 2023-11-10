import cmd
import glob
import inspect
import json
import os
import socket
import readline

from argument import (
    Argument,
    CHECK_ARGC,
    FORMAT_ARGUMENTS,
    get_argument_string,
)

BUFFER_SIZE = 1024
INTRO_CHAR = "="
UNIX_DOMAIN_SOCKET_PATH = "/tmp/taskmaster.sock"

RESET = "\033[0m"
BOLD = "\033[1m"
GREEN = "\033[92m"
CYAN = "\033[96m"

PROMPT_START_IGNORE = "\001"
PROMPT_END_IGNORE = "\002"


def communicate(message):
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
            try:
                s.connect(UNIX_DOMAIN_SOCKET_PATH)
            except FileNotFoundError:
                print(f"Socket {UNIX_DOMAIN_SOCKET_PATH} not found")
                return
            except Exception as e:
                print(f"Failed to connect to taskmasterd: {e}")
                return
            try:
                s.sendall(message.encode())
            except Exception as e:
                print(f"Failed to write to taskmasterd: {e}")
                return
            needs_newline = False
            while True:
                try:
                    part = s.recv(BUFFER_SIZE).decode()
                except KeyboardInterrupt:
                    print()
                    return
                except Exception as e:
                    print(f"Failed to read from taskmasterd: {e}")
                    return
                if not part:
                    break
                print(part, flush=True, end="")
                if not part.endswith("\n"):
                    needs_newline = True
                else:
                    needs_newline = False
            if needs_newline:
                print()
            elif message == '"Shutdown"':
                print("Shutdown successful")
    except Exception as e:
        print(f"Unknown error: {e}")


def process_cmd(arg, expected_argument):
    current_frame = inspect.currentframe()
    calling_frame = current_frame.f_back
    method_name = calling_frame.f_code.co_name
    command = method_name[3:]
    argv = arg.split()
    argc = len(argv)
    if CHECK_ARGC[expected_argument](argc):
        message = FORMAT_ARGUMENTS[expected_argument](command.title(), argc, argv)
        if message is not None:
            communicate(json.dumps(message))
    else:
        print(f"{command} {get_argument_string(expected_argument)}")
        class_name = calling_frame.f_locals["self"].__class__.__name__
        method = getattr(eval(class_name, calling_frame.f_globals), method_name, None)
        print(method.__doc__)


def input_swallowing_interrupt(_input):
    def _input_swallowing_interrupt(*args):
        try:
            return _input(*args)
        except KeyboardInterrupt:
            print("^C")
            return "\n"

    return _input_swallowing_interrupt


class TaskMasterShell(cmd.Cmd):
    prompt = f"{PROMPT_START_IGNORE}{BOLD}{CYAN}{PROMPT_END_IGNORE}taskmaster>{PROMPT_START_IGNORE}{RESET}{PROMPT_END_IGNORE} "

    def cmdloop(self, *args, **kwargs):
        old_input_fn = cmd.__builtins__["input"]
        cmd.__builtins__["input"] = input_swallowing_interrupt(old_input_fn)
        try:
            super().cmdloop(*args, **kwargs)
        finally:
            cmd.__builtins__["input"] = old_input_fn

        self.old_completer = readline.get_completer()
        readline.set_completer(self.complete)
        readline.parse_and_bind(self.completekey + ": complete")
        old_delims = readline.get_completer_delims()
        readline.set_completer_delims(old_delims.replace("-", ""))

    def emptyline(self):
        pass

    def default(self, arg):
        if arg == "EOF":
            print()
            return True
        else:
            print(f"{arg.split()[0]}: command not found")

    def do_exit(self, _):
        """exit : Exit the taskmaster shell"""
        return True

    do_quit = do_exit

    def do_clear(self, arg):
        """clear <name> : Clear the log files of a process"""
        process_cmd(arg, Argument.ONE)

    def do_config(self, arg):
        """config <name> : Get the task configuration in json"""
        process_cmd(arg, Argument.ONE)

    def do_http(self, arg):
        """http enable <port> : Enable http logging\nhttp disable       : Disable http logging\nhttp status        : Show http logging status"""
        process_cmd(arg, Argument.HTTP)

    def do_maintail(self, arg):
        """maintail     : complete taskmasterd main log file\nmaintail N   : last N lines of taskmasterd main log file\nmaintail f   : complete and continuous taskmasterd main log file\nmaintail f42 : last N lines of taskmasterd main log file, continuously"""
        process_cmd(arg, Argument.MAINTAIL)

    def do_restart(self, arg):
        """restart <name> : Restart a process"""
        process_cmd(arg, Argument.ZERO_TO_TWO)

    def do_shutdown(self, arg):
        """shutdown : Shut the remote taskmasterd down."""
        process_cmd(arg, Argument.ZERO)

    def do_signal(self, arg):
        """signal <signum or signame> <taskname>       : Signal a task group\nsignal <signum or signame> <taskname> <idx> : Signal a task"""
        process_cmd(arg, Argument.SIGNAL)

    def do_start(self, arg):
        """start <name> : Start a process"""
        process_cmd(arg, Argument.ZERO_TO_TWO)

    def do_stop(self, arg):
        """stop <name> : Stop a process"""
        process_cmd(arg, Argument.ZERO_TO_TWO)

    def do_status(self, arg):
        "status        : Get all process status info\nstatus <name> : Get status for a single process"
        process_cmd(arg, Argument.OPTIONAL_STRING)

    def do_tail(self, arg):
        """tail <taskname> <stdout or stderr>     : complete taskmasterd main log file\ntail <taskname> <stdout or stderr> N   : last N lines of taskmasterd main log file\ntail <taskname> <stdout or stderr> f   : complete and continuous taskmasterd main log file\ntail <taskname> <stdout or stderr> f42 : last N lines of taskmasterd main log file, continuously"""
        process_cmd(arg, Argument.TAIL)

    def do_update(self, arg):
        """update <filename> : Reload the config file and add/remove tasks as necessary"""
        process_cmd(arg, Argument.OPTIONAL_STRING)

    def complete_update(self, text, line, *_):
        mline = line.partition(" ")[2]
        offs = len(mline) - len(text)
        return [
            fp[offs:] + "/" if os.path.isdir(fp) else fp[offs:]
            for fp in glob.glob(mline + "*")
        ]


if __name__ == "__main__":
    width = os.get_terminal_size().columns
    top_line = INTRO_CHAR * width
    middle_line = "  WELCOME TO TASKMASTER  ".center(width, INTRO_CHAR)
    intro = f"{BOLD}{GREEN}{top_line}\n{middle_line}\n{top_line}{RESET}"
    TaskMasterShell().cmdloop(intro)
    print("Goodbye.")
