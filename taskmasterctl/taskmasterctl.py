# TODO: empty line

import cmd
import json
import socket


def send_to_socket(message):
    print(message)
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
            s.connect("/tmp/.unixdomain.sock")
            s.sendall(message.encode())
            response = s.recv(1024)
            print("Received:", response.decode())
    except Exception as e:
        print(str(e))


def input_swallowing_interrupt(_input):
    def _input_swallowing_interrupt(*args):
        try:
            return _input(*args)
        except KeyboardInterrupt:
            print("^C")
            return "\n"

    return _input_swallowing_interrupt


class TaskMasterShell(cmd.Cmd):
    intro = "=== WELCOME TO TASKMASTER ==="
    prompt = "taskmaster> "

    def cmdloop(self, *args, **kwargs):
        old_input_fn = cmd.__builtins__["input"]
        cmd.__builtins__["input"] = input_swallowing_interrupt(old_input_fn)
        try:
            super().cmdloop(*args, **kwargs)
        finally:
            cmd.__builtins__["input"] = old_input_fn

    def empty_line(self):
        print(self.prompt, end="")
        return False

    def default(self, arg):
        if arg == "EOF":
            print()
            return True
        else:
            print(f"{arg}: command not found")

    def do_exit(self, arg):
        """Exit the taskmasterctl shell."""
        return True

    def do_help(self, arg):
        print("HELP")

    def do_config(self, arg):
        send_to_socket(json.dumps({"Config": arg}))

    def do_start(self, arg):
        send_to_socket(json.dumps({"Start": arg}))

    def do_shutdown(self, arg):
        send_to_socket(json.dumps("Shutdown"))

    def do_stop(self, arg):
        send_to_socket(json.dumps({"Stop": arg}))

    def do_status(self, arg):
        send_to_socket(json.dumps({"Status": arg or None}))

    def do_update(self, arg):
        print(arg)
        send_to_socket(json.dumps({"Update": arg}))


if __name__ == "__main__":
    TaskMasterShell().cmdloop()
    print("Goodbye.")
