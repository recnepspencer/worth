"""Link, validate, then publish an arm64 Darwin artifact; never launch or retry it."""
import os
from pathlib import Path
import errno
import subprocess
import sys

from .macho import InvalidImage, read
from .response import expanded, ResponseFileError
from .placement import (UnsafeLayout, linkedit_arguments, relocated_arguments,
                        unsafe_segments, validate)


def executable_link(arguments):
    tokens = [part for arg in arguments for part in arg.split(',')]
    return not any(flag in tokens for flag in ('-dynamiclib', '-shared', '-bundle', '-r'))


def link(arguments, compiler='cc'):
    args = expanded(arguments)
    if not executable_link(args):
        return subprocess.call([compiler, *arguments])
    if '-o' not in args:
        raise UnsafeLayout('executable link must name its output with -o')
    output_index = args.index('-o') + 1
    destination = Path(args[output_index])
    stage = destination.with_name(destination.name + '.layout-pending')
    # Keep every original @file intact. cc's last -o wins, including when
    # the original destination was inside a response file.
    staged = [*arguments, '-o', str(stage)]
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
    except (OSError, ResponseFileError) as error:
        detail = 'OS argument limit exceeded; original response files were forwarded' \
            if isinstance(error, OSError) and error.errno == errno.E2BIG else str(error)
        sys.stderr.write(f'MACOS_LINK_COMMAND: {detail}\n')
        return 1
    except (InvalidImage, UnsafeLayout, ValueError) as error:
        sys.stderr.write(f'MACOS_EXEC_LAYOUT: {error}\n'
                         'arm64 Darwin executable refused before launch; see '
                         'scripts/macos_link/README.md (#67). No retry was attempted.\n')
        return 1
