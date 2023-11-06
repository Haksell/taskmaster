#!/opt/homebrew/bin/python3

import signal
import os


def sighup_handler(signum, frame):
    print(f"Received SIGHUP (signal {signum})", flush=True)


signal.signal(signal.SIGHUP, sighup_handler)

print(f"PID: {os.getpid()}", flush=True)

while True:
    pass
