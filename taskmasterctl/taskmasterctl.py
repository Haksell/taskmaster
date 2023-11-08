import cmd
from enum import Enum, auto
import glob
import inspect
import json
import os
import socket
import readline
import signal

BUFFER_SIZE = 1024
INTRO_CHAR = "="
UNIX_DOMAIN_SOCKET_PATH = "/tmp/taskmaster.sock"

RESET = "\033[0m"
BOLD = "\033[1m"
GREEN = "\033[92m"
CYAN = "\033[96m"

PROMPT_START_IGNORE = "\001"
PROMPT_END_IGNORE = "\002"


class Argument(Enum):
    MAINTAIL = auto()
    ONE = auto()
    OPTIONAL_POSITIVE = auto()
    OPTIONAL_STRING = auto()
    SIGNAL = auto()
    TAIL = auto()
    ZERO = auto()
    ZERO_TO_TWO = auto()


CHECK_ARGC = {
    Argument.MAINTAIL: lambda argc: argc <= 1,
    Argument.ONE: lambda argc: argc == 1,
    Argument.OPTIONAL_POSITIVE: lambda argc: argc <= 1,
    Argument.OPTIONAL_STRING: lambda argc: argc <= 1,
    Argument.SIGNAL: lambda argc: argc == 2,
    Argument.TAIL: lambda argc: 2 <= argc <= 3,
    Argument.ZERO_TO_TWO: lambda argc: argc <= 2,
    Argument.ZERO: lambda argc: argc == 0,
}

ARGUMENT_STRING = {
    Argument.MAINTAIL: "usage:",
    Argument.ONE: "requires exactly one argument",
    Argument.OPTIONAL_POSITIVE: "accepts zero or one unsigned integer argument",
    Argument.OPTIONAL_STRING: "accepts zero or one argument",
    Argument.SIGNAL: "requires a signal number or name, followed by a task name",
    Argument.TAIL: "usage:",
    Argument.ZERO: "doesn't accept an argument",
    Argument.ZERO_TO_TWO: "requires zero, one or two arguments",
}


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


def handle_zero_to_two_arguments(command, argc, argv):
    if argc == 0:
        return {command: None}
    elif argc == 1:
        return {command: [argv[0], None]}
    else:
        try:
            idx = int(argv[1])
            assert idx >= 0
        except (AssertionError, ValueError):
            print(f'Invalid index: "{argv[1]}"')
            return None
        return {command: [argv[0], idx]}


def handle_optional_positive(command, argc, argv):
    if argc == 0:
        return {command: None}
    else:
        try:
            val = int(argv[0])
            assert val > 0
        except (AssertionError, ValueError):
            print(f'"{argv[0]}" is not a positive number')
            return None
        return {command: val}


def handle_signal_arguments(command, argv):
    def get_signum(sigstr):
        try:
            n = int(sigstr)
            if 0 <= n <= 255:
                return n
        except ValueError:
            sigstr = sigstr.upper()
            if not sigstr.startswith("SIG"):
                sigstr = "SIG" + sigstr
            try:
                return getattr(signal, sigstr).value
            except AttributeError:
                pass

    signum = get_signum(argv[0])
    if signum is None:
        print(f'"{argv[0]}" is not a valid signal')
        return None
    return {command: [signum, argv[1]]}


def get_tail_type(arg):
    if arg.startswith("f"):
        type_string = "Stream"
        arg = arg[1:]
    else:
        type_string = "Fixed"
    if arg == "":
        return {type_string: None}
    if all(c == "0" for c in arg):
        print(f"Can't request 0 line from taskmasterd")
        return None
    try:
        num_lines = int(arg)
        return {type_string: num_lines}
    except ValueError:
        print(f'"{arg}" is not a valid number of lines')
        return None


def handle_maintail_argument(command, arg):
    tail_type = get_tail_type(arg)
    if tail_type is None:
        return None
    else:
        return {command: tail_type}


def handle_tail_argument(command, argc, argv):
    task_name = argv[0]
    output_type = argv[1].title()
    if output_type != "Stdout" and output_type != "Stderr":
        print(f'Invalid output type: "{argv[1]}"')
        return None
    tail_type = get_tail_type("" if argc == 2 else argv[2])
    if tail_type is None:
        return None
    return {command: [task_name, output_type, tail_type]}


def process_cmd(arg, expected_argument):
    current_frame = inspect.currentframe()
    calling_frame = current_frame.f_back
    method_name = calling_frame.f_code.co_name
    argv = arg.split()
    argc = len(argv)
    if CHECK_ARGC[expected_argument](argc):
        command = method_name[3:].title()
        message = (
            command
            if expected_argument == Argument.ZERO
            else handle_zero_to_two_arguments(command, argc, argv)
            if expected_argument == Argument.ZERO_TO_TWO
            else handle_optional_positive(command, argc, argv)
            if expected_argument == Argument.OPTIONAL_POSITIVE
            else handle_signal_arguments(command, argv)
            if expected_argument == Argument.SIGNAL
            else handle_maintail_argument(command, argv[0] if argv else "")
            if expected_argument == Argument.MAINTAIL
            else handle_tail_argument(command, argc, argv)
            if expected_argument == Argument.TAIL
            else {command: arg or None}
        )
        if message is None:
            return
        communicate(json.dumps(message))
    else:
        argument_string = ARGUMENT_STRING[expected_argument]
        print(f"{method_name[3:]} {argument_string}")
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

    def do_config(self, arg):
        """config <name> : Get the task configuration in json"""
        process_cmd(arg, Argument.ONE)

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
        """signal <signum or signame> <taskname> : Signal a process"""
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
        """tail <stdout or stderr> <taskname>     : complete taskmasterd main log file\ntail <stdout or stderr> <taskname> N   : last N lines of taskmasterd main log file\ntail <stdout or stderr> <taskname> f   : complete and continuous taskmasterd main log file\ntail <stdout or stderr> <taskname> f42 : last N lines of taskmasterd main log file, continuously"""
        process_cmd(arg, Argument.TAIL)

    def do_update(self, arg):
        """update <filename> : Reload the config file and add/remove tasks as necessary"""
        process_cmd(arg, Argument.OPTIONAL_STRING)

    def complete_update(self, text, line, *_):
        mline = line.partition(" ")[2]
        offs = len(mline) - len(text)
        return [fp[offs:] for fp in glob.glob(mline + "*")]


if __name__ == "__main__":
    width = os.get_terminal_size().columns
    top_line = INTRO_CHAR * width
    middle_line = "  WELCOME TO TASKMASTER  ".center(width, INTRO_CHAR)
    TaskMasterShell().cmdloop(
        f"{BOLD}{GREEN}{top_line}\n{middle_line}\n{top_line}{RESET}"
    )
    print("Goodbye.")
