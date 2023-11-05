# TODO: parse action.rs to avoid code duplication

import cmd
import glob
import json
import socket
import readline
import sys

BUFFER_SIZE = 1024

UNIX_DOMAIN_SOCKET = "/tmp/.unixdomain.sock"

RESET = "\033[0m"
BOLD = "\033[1m"
RED = "\033[91m"
GREEN = "\033[92m"
CYAN = "\033[96m"


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
            response = b"".join(response_parts)
            response = response.decode().rstrip()
            if response:
                print(response)
            elif message == "Shutdown":
                print("Shutdown successful")
    except Exception as e:
        print_error(f"Unknown error: {e}")


def input_swallowing_interrupt(_input):
    def _input_swallowing_interrupt(*args):
        try:
            return _input(*args)
        except KeyboardInterrupt:
            print("^C")
            return "\n"

    return _input_swallowing_interrupt


class TaskMasterShell(cmd.Cmd):
    intro = f"""{BOLD}{GREEN}\
=============================
=== WELCOME TO TASKMASTER ===
============================={RESET}"""
    prompt = f"{BOLD}{CYAN}taskmaster>{RESET} "

    def cmdloop(self, *args, **kwargs):
        old_input_fn = cmd.__builtins__["input"]
        cmd.__builtins__["input"] = input_swallowing_interrupt(old_input_fn)
        try:
            super().cmdloop(*args, **kwargs)
        finally:
            cmd.__builtins__["input"] = old_input_fn

        try:
            self.old_completer = readline.get_completer()
            readline.set_completer(self.complete)
            readline.parse_and_bind(self.completekey + ": complete")
            old_delims = readline.get_completer_delims()
            readline.set_completer_delims(old_delims.replace("-", ""))
        except ImportError:
            pass

    def emptyline(self):
        pass

    def default(self, arg):
        if arg == "EOF":
            print()
            return True
        else:
            print(f"{arg.split()[0]}: command not found")

    def do_exit(self, arg):
        """exit : Exit the taskmaster shell"""
        return True

    def do_config(self, arg):
        """config <name> : Get the task configuration in json"""
        send_to_socket(json.dumps({"Config": arg}))

    def do_shutdown(self, arg):
        """shutdown : Shut the remote taskmasterd down."""
        send_to_socket(json.dumps("Shutdown"))

    def do_start(self, arg):
        """start <name> : Start a process"""
        send_to_socket(json.dumps({"Start": arg}))

    def do_stop(self, arg):
        """stop <name> : Stop a process"""
        send_to_socket(json.dumps({"Stop": arg}))

    def do_status(self, arg):
        "status        : Get all process status info\nstatus <name> : Get status for a single process"
        send_to_socket(json.dumps({"Status": arg or None}))

    def do_update(self, arg):
        """update <filename> : Reload the config file and add/remove tasks as necessary"""
        print(arg)
        send_to_socket(json.dumps({"Update": arg}))

    def complete_update(self, text, line, start_index, end_index):
        mline = line.partition(" ")[2]
        offs = len(mline) - len(text)
        return [fp[offs:] for fp in glob.glob(mline + "*")]


if __name__ == "__main__":
    TaskMasterShell().cmdloop()
    print("Goodbye.")
