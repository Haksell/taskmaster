from enum import Enum, auto
import signal


class Argument(Enum):
    HTTP = auto()
    MAINTAIL = auto()
    ONE = auto()
    OPTIONAL_POSITIVE = auto()
    OPTIONAL_STRING = auto()
    SIGNAL = auto()
    TAIL = auto()
    ZERO = auto()
    ZERO_TO_TWO = auto()


CHECK_ARGC = {
    Argument.HTTP: lambda argc: 1 <= argc <= 2,
    Argument.MAINTAIL: lambda argc: argc <= 1,
    Argument.ONE: lambda argc: argc == 1,
    Argument.OPTIONAL_POSITIVE: lambda argc: argc <= 1,
    Argument.OPTIONAL_STRING: lambda argc: argc <= 1,
    Argument.SIGNAL: lambda argc: 2 <= argc <= 3,
    Argument.TAIL: lambda argc: 2 <= argc <= 3,
    Argument.ZERO: lambda argc: argc == 0,
    Argument.ZERO_TO_TWO: lambda argc: argc <= 2,
}

ARGUMENT_STRING = {
    Argument.ONE: "requires exactly one argument",
    Argument.OPTIONAL_POSITIVE: "accepts zero or one unsigned integer argument",
    Argument.OPTIONAL_STRING: "accepts zero or one argument",
    Argument.SIGNAL: "requires a signal number or name, followed by a task name and an optional index",
    Argument.ZERO: "doesn't accept an argument",
    Argument.ZERO_TO_TWO: "requires zero, one or two arguments",
}


def get_argument_string(argument):
    return ARGUMENT_STRING.get(argument, "usage:")


def parse_index(s):
    try:
        idx = int(s)
        assert idx >= 0
        return idx
    except (AssertionError, ValueError):
        print(f'Invalid index: "{s}"')
        return None


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


def format_arguments_http(command, argc, argv):
    command = "HttpLogging"
    get_status_command = "GetHttpLoggingStatus"
    if argv[0] == "enable":
        if argc == 1:
            print("http enabling requires a port")
            return None
        try:
            port = int(argv[1])
            assert 0 <= port <= 65535
            return {command: port}
        except (AssertionError, ValueError):
            print(f'"{argv[1]}" is not a valid port')
            return None
    elif argv[0] == "disable":
        if argc == 2:
            print("http disabling does not accept an argument")
            return None
        else:
            return {command: None}
    elif argv[0] == "status":
        if argc == 2:
            print("http status does not accept an argument")
            return None
        else:
            return {get_status_command: None}

    else:
        print(f"http: unknown keyword: {argv[0]}")
        return None


def format_arguments_maintail(command, argc, argv):
    arg = argv[0] if argc == 1 else ""
    tail_type = get_tail_type(arg)
    if tail_type is None:
        return None
    else:
        return {command: tail_type}


def format_arguments_one(command, argc, argv):
    return {command: argv[0]}


def format_arguments_optional_positive(command, argc, argv):
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


def format_arguments_optional_string(command, argc, argv):
    return {command: argv[0] if argc == 1 else None}


def format_arguments_signal(command, argc, argv):
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

    if argc == 2:
        return {command: [signum, argv[1], None]}
    else:
        idx = parse_index(argv[2])
        if idx is None:
            return None
        return {command: [signum, argv[1], idx]}


def format_arguments_tail(command, argc, argv):
    task_name = argv[0]
    output_type = argv[1].title()
    if output_type != "Stdout" and output_type != "Stderr":
        print(f'Invalid output type: "{argv[1]}"')
        return None
    tail_type = get_tail_type("" if argc == 2 else argv[2])
    if tail_type is None:
        return None
    return {command: [task_name, output_type, tail_type]}


def format_arguments_zero(command, argc, argv):
    return command


def format_arguments_zero_to_two(command, argc, argv):
    if argc == 0:
        return {command: None}
    elif argc == 1:
        return {command: [argv[0], None]}
    else:
        idx = parse_index(argv[1])
        return None if idx is None else {command: [argv[0], idx]}


FORMAT_ARGUMENTS = {
    Argument.HTTP: format_arguments_http,
    Argument.MAINTAIL: format_arguments_maintail,
    Argument.ONE: format_arguments_one,
    Argument.OPTIONAL_POSITIVE: format_arguments_optional_positive,
    Argument.OPTIONAL_STRING: format_arguments_optional_string,
    Argument.SIGNAL: format_arguments_signal,
    Argument.TAIL: format_arguments_tail,
    Argument.ZERO: format_arguments_zero,
    Argument.ZERO_TO_TWO: format_arguments_zero_to_two,
}
