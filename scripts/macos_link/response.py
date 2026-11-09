"""Inspect GNU response tokens without changing the argv forwarded to cc."""
from pathlib import Path


class ResponseFileError(ValueError):
    pass


def tokens(source):
    # LLVM TokenizeGNUCommandLine: backslash escapes the next character,
    # including inside either quote kind (unlike POSIX shlex single quotes).
    # https://github.com/llvm/llvm-project/blob/main/llvm/lib/Support/CommandLine.cpp
    result, token, quoted, active = [], [], None, False
    index = 0
    while index < len(source):
        char = source[index]
        if char == '\\' and index + 1 < len(source):
            index += 1
            token.append(source[index])
            active = True
        elif quoted:
            if char == quoted:
                quoted = None
            else:
                token.append(char)
        elif char in ('"', "'"):
            quoted, active = char, True
        elif char in ' \t\r\n\v\f':
            if active:
                result.append(''.join(token))
                token, active = [], False
        else:
            token.append(char)
            active = True
        index += 1
    if active:
        result.append(''.join(token))
    return result


def expanded(arguments, ancestors=()):
    result = []
    for argument in arguments:
        if argument.startswith('@'):
            path = Path(argument[1:]).resolve()
            if path in ancestors:
                raise ResponseFileError(f'cyclic response file: {path}')
            result.extend(expanded(tokens(path.read_text()), (*ancestors, path)))
        else:
            result.append(argument)
    return result
