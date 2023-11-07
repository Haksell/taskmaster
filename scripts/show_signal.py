#!/usr/bin/env python3

import signal
import os
import sys


def universal_signal_handler(signum, frame):
    print(f"PID {os.getpid()}: Received signal {signum}", flush=True)
    if signum == signal.SIGTERM:
        print(f"PID {os.getpid()}: Goodbye.", flush=True)
        sys.exit(0)


UNCATCHABLE = [signal.SIGKILL, signal.SIGSTOP, signal.SIGWINCH]

for i in [x for x in dir(signal) if x.startswith("SIG") and not x.startswith("SIG_")]:
    try:
        signum = getattr(signal, i)
        if signum not in UNCATCHABLE:
            signal.signal(signum, universal_signal_handler)
    except (ValueError, AttributeError, RuntimeError) as e:
        print(f"PID {os.getpid()}: Signal {i} cannot be caught.", flush=True)
    except Exception as e:
        print(
            f"PID {os.getpid()}: An error occurred while setting handler for {i}: {e}",
            flush=True,
        )

print(f"PID: {os.getpid()}", flush=True)
while True:
    signal.pause()
