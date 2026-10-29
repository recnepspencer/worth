"""Run a lane once, retaining its exact command and source revision."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
import sys


def run(command, log, cwd=None):
    cwd = Path(cwd or Path.cwd()).resolve()
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cwd, text=True).strip()
    clean = not subprocess.check_output(['git', 'status', '--short'], cwd=cwd, text=True).strip()
    start = datetime.now(timezone.utc).isoformat()
    log = Path(log)
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open('w', encoding='utf-8') as stream:
        stream.write(f'Command: {json.dumps(command)}\nWorking directory: {cwd}\n'
                     f'HEAD: {head}\nGit status --short empty: {clean}\nStart: {start}\n\n')
        stream.flush()
        try:
            child = subprocess.Popen(command, cwd=cwd, stdout=subprocess.PIPE,
                                     stderr=subprocess.STDOUT)
            for line in child.stdout:
                value = line.decode('utf-8', errors='replace')
                stream.write(value)
                stream.flush()
                print(value, end='', flush=True)
            child.stdout.close()
            code = child.wait()
        except OSError as error:
            stream.write(f'Launch failed: {error}\n')
            code = 127
        stream.write(f'\nEnd: {datetime.now(timezone.utc).isoformat()}\nExit code: {code}\n')
    return code


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--log', required=True, type=Path)
    parser.add_argument('--cwd', type=Path)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('a command is required')
    return run(command, args.log, args.cwd)


if __name__ == '__main__':
    sys.exit(main())
