"""Link, validate, then publish an arm64 Darwin artifact; never launch or retry it."""
import os
from pathlib import Path
import shlex
import subprocess
import sys

from .macho import InvalidImage, read
from .placement import (UnsafeLayout, linkedit_arguments, relocated_arguments,
                        unsafe_segments, validate)


def expanded(arguments):
    result = []
    for argument in arguments:
        if argument.startswith('@'):
            result.extend(expanded(shlex.split(Path(argument[1:]).read_text())))
        else:
            result.append(argument)
    return result


def executable_link(arguments):
    tokens = [part for arg in arguments for part in arg.split(',')]
    return not any(flag in tokens for flag in ('-dynamiclib', '-shared', '-bundle', '-r'))


def link(arguments, compiler='/usr/bin/cc'):
    args = expanded(arguments)
    if not executable_link(args):
        return subprocess.call([compiler, *arguments])
    if '-o' not in args:
        raise UnsafeLayout('executable link must name its output with -o')
    output_index = args.index('-o') + 1
    destination = Path(args[output_index])
    stage = destination.with_name(destination.name + '.layout-pending')
    staged = args.copy()
    staged[output_index] = str(stage)
    try:
        code = subprocess.call([compiler, *staged, *linkedit_arguments()])
        if code:
            return code
        initial = read(stage)
        if initial.kind != 2:
            raise UnsafeLayout('unexpected non-executable output')
        if unsafe_segments(initial):
            placement = relocated_arguments(initial)
            # This is a planned geometry/emit stage, not a failed-link retry.
            code = subprocess.call([compiler, *staged, *placement])
            if code:
                return code
        final = read(stage)
        validate(final)
        if final.flags != initial.flags:
            raise UnsafeLayout('link placement changed Mach-O flags')
        pagezero = lambda image: tuple(s for s in image.segments if s.name == '__PAGEZERO')
        if pagezero(final) != pagezero(initial):
            raise UnsafeLayout('link placement changed page-zero protection')
        os.replace(stage, destination)
        return 0
    finally:
        stage.unlink(missing_ok=True)


def main():
    try:
        return link(sys.argv[1:])
    except (OSError, InvalidImage, UnsafeLayout, ValueError) as error:
        sys.stderr.write(f'MACOS_EXEC_LAYOUT: {error}\n'
                         'arm64 Darwin executable refused before launch; see '
                         'scripts/macos_link/README.md (#67). No retry was attempted.\n')
        return 1
