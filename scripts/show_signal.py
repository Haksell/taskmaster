import signal
import os

# Define a signal handler for SIGTERM
def sigterm_handler(signum, frame):
    print(f"Received SIGTERM (signal {signum})")
    exit(0)

# Define a signal handler for SIGHUP
def sighup_handler(signum, frame):
    print(f"Received SIGHUP (signal {signum})")

# Define a signal handler for a custom signal (e.g., SIGUSR1)
def custom_handler(signum, frame):
    print(f"Received custom signal (signal {signum})")

# Set up signal handlers for SIGTERM, SIGHUP, and a custom signal (e.g., SIGUSR1)
signal.signal(signal.SIGTERM, sigterm_handler)
signal.signal(signal.SIGHUP, sighup_handler)

# You can use a custom signal number (e.g., 10 for SIGUSR1)
custom_signal = 10
signal.signal(custom_signal, custom_handler)

print(f"PID: {os.getpid()}")

# Keep the program running
while True:
    pass

