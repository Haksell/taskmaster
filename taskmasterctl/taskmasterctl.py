# TODO: parse action.rs to avoid code duplication

import cmd
from enum import Enum
import glob
import inspect
import json
import os
import socket
import readline
import sys

BUFFER_SIZE = 1024
INTRO_CHAR = "="
UNIX_DOMAIN_SOCKET = "/tmp/.unixdomain.sock"

RESET = "\033[0m"
BOLD = "\033[1m"
RED = "\033[91m"
GREEN = "\033[92m"
CYAN = "\033[96m"

PROMPT_START_IGNORE = "\001"
PROMPT_END_IGNORE = "\002"


class Argument(Enum):
    ZERO = "no"
    OPTIONAL = "zero or one"
    ONE = "one"


def print_error(s):
    print(f"{RED}{s}{RESET}", file=sys.stderr)


def send_to_socket(message):
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
            try:
                s.connect(UNIX_DOMAIN_SOCKET)
            except FileNotFoundError:
                print_error(f"Socket {UNIX_DOMAIN_SOCKET} not found")
                return
            except Exception as e:
                print_error(f"Failed to connect to taskmasterd: {e}")
                return
            try:
                s.sendall(message.encode())
            except Exception as e:
                print_error(f"Failed to write to taskmasterd: {e}")
                return
            response_parts = []
            while True:
                try:
                    part = s.recv(BUFFER_SIZE)
                except Exception as e:
                    print_error(f"Failed to read from taskmasterd: {e}")
                    return
                if not part:
                    break
                response_parts.append(part)
            response = b"".join(response_parts).decode().rstrip()
            if response:
                print(response)
            elif message == "Shutdown":
                print("Shutdown successful")
    except Exception as e:
        print_error(f"Unknown error: {e}")


def process_cmd(arg, expected_argument):
    current_frame = inspect.currentframe()
    calling_frame = current_frame.f_back
    method_name = calling_frame.f_code.co_name
    argc = len(arg.split())
    if (
        expected_argument == Argument.ZERO
        and argc == 0
        or expected_argument == Argument.OPTIONAL
        and argc <= 1
        or expected_argument == Argument.ONE
        and argc == 1
    ):
        command = method_name[3:].title()
        message = json.dumps(
            command
            if expected_argument == Argument.ZERO
            else {command: arg or None}
            if expected_argument == Argument.OPTIONAL
            else {command: arg}
        )
        send_to_socket(message)
    else:
        print_error(f"{method_name[3:]} requires {expected_argument.value} argument")
        class_name = (
            calling_frame.f_locals.get("self", None).__class__.__name__
            if "self" in calling_frame.f_locals
            else None
        )
        if class_name:
            method = getattr(
                eval(class_name, calling_frame.f_globals), method_name, None
            )
        else:
            method = calling_frame.f_globals.get(method_name)
        print(inspect.getdoc(method))


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

    def do_exit(self, arg):
        """exit: Exit the taskmaster shell"""
        return True

    do_quit = do_exit

    def do_config(self, arg):
        """config <name>: Get the task configuration in json"""
        process_cmd(arg, Argument.ONE)

    def do_shutdown(self, arg):
        """shutdown: Shut the remote taskmasterd down."""
        process_cmd(arg, Argument.ZERO)

    def do_start(self, arg):
        """start <name>: Start a process"""
        process_cmd(arg, Argument.ONE)

    def do_stop(self, arg):
        """stop <name>: Stop a process"""
        process_cmd(arg, Argument.ONE)

    def do_status(self, arg):
        "status       : Get all process status info\nstatus <name>: Get status for a single process"
        process_cmd(arg, Argument.OPTIONAL)

    def do_update(self, arg):
        """update <filename>: Reload the config file and add/remove tasks as necessary"""
        process_cmd(arg, Argument.ONE)

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
