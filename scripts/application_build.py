#!/usr/bin/env python3
"""Issue one embedded application build identity, then run the build command.

    python scripts/application_build.py cargo build --locked

Every invocation issues a fresh UUID, including when Cargo would otherwise be
fresh. Applications embed it with env!("WORTH_IMPLEMENTATION_BUILD_ID"); it is
descriptive build input, never runtime authorization or a startup override.
"""
import argparse
import os
import subprocess
import uuid


def application_build_environment(parent):
    """Preserve the caller's environment and replace any previous build ID."""
    return {**parent, "WORTH_IMPLEMENTATION_BUILD_ID": str(uuid.uuid4())}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("command", nargs=argparse.REMAINDER,
                        help="build command and its original arguments")
    command = parser.parse_args(argv).command
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        parser.error("supply the application build command, for example: cargo build --locked")
    return subprocess.run(command, env=application_build_environment(os.environ)).returncode


if __name__ == "__main__":
    raise SystemExit(main())
