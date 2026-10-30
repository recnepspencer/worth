"""The runner's single Cargo process boundary and compiler-JSON capture."""
from datetime import datetime, timezone
import json
import hashlib
import re
from pathlib import Path
import subprocess
import shutil
import sys


def instrument(command):
    command = list(command)
    if command[:1] != ['cargo']:
        return command
    if len(command) < 2:
        raise ValueError('Cargo invocation has no subcommand')
    subcommand = command[1]
    if subcommand in ('build', 'check', 'test', 'rustc', 'clippy', 'run'):
        boundary = command.index('--') if '--' in command else len(command)
        if any(a.startswith('--message-format') for a in command[:boundary]):
            index = next(i for i, a in enumerate(command[:boundary]) if a.startswith('--message-format'))
            value = command[index].partition('=')[2] or command[index + 1]
            if not value.startswith('json'):
                raise ValueError('compiler output must be Cargo JSON')
        else:
            command.insert(2, '--message-format=json')
    elif subcommand == 'nextest':
        if '--binaries-metadata' not in command and not any(a.startswith('--cargo-message-format') for a in command):
            command.insert(3, '--cargo-message-format=json')
    elif subcommand not in ('metadata', 'tree', 'fmt', 'llvm-lines'):
        raise ValueError(f'uncaptured Cargo subcommand: {subcommand}')
    return command


def capture_record(environment, path):
    environment = environment if environment is not None else __import__('os').environ
    if index := environment.get('WORTH_TEST_COST_CAPTURE_INDEX'):
        index = Path(index)
        index.parent.mkdir(parents=True, exist_ok=True)
        with index.open('ab', buffering=0) as stream:
            stream.write((json.dumps({'log': str(Path(path).resolve())}) + '\n').encode('utf-8'))


def retention_name(relative, head, start):
    relative = str(relative).replace('\\', '/')
    label = re.sub(r'[^A-Za-z0-9_.-]', '-', relative)[:64]
    stamp = ''.join(character for character in start if character.isdigit())
    digest = hashlib.sha256(relative.encode()).hexdigest()[:12]
    return f'{label}-{head[:12]}-{stamp}-{digest}.log'


class CargoRunner:
    def __init__(self, repository, environment=None):
        for stream in (sys.stdout, sys.stderr):
            if hasattr(stream, 'reconfigure'):
                stream.reconfigure(encoding='utf-8', errors='replace')
        self.repository = Path(repository).resolve()
        self.environment = environment
        self.logs = []
        self.target_directories = {}

    def __call__(self, command, cwd, log_path):
        command = instrument(command)
        cwd, log_path = Path(cwd).resolve(), Path(log_path)
        log_path.parent.mkdir(parents=True, exist_ok=True)
        head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.repository, text=True).strip()
        clean = not subprocess.check_output(['git', 'status', '--short'], cwd=self.repository, text=True).strip()
        start = datetime.now(timezone.utc).isoformat()
        ordered = command[:2] == ['cargo', 'test'] and '--doc' in command
        marker = {'type': 'cargo-invocation', 'command': command, 'cwd': str(cwd),
                  'capture': 'ordered' if ordered else 'separate',
                  'mode': 'check' if command[1:2] in (['check'], ['clippy']) else 'build',
                  'target_dir': None,
                  'cargo_profile': ('release' if '--release' in command else
                                    'test' if command[1:2] in (['test'], ['nextest']) else 'dev')}
        for flag in ('--profile', '--cargo-profile'):
            if flag in command:
                marker['cargo_profile'] = command[command.index(flag) + 1]
        if '--target-dir' in command:
            marker['target_dir'] = command[command.index('--target-dir') + 1]
        else:
            matches = [(root, target) for root, target in self.target_directories.items()
                       if cwd.is_relative_to(root)]
            if matches:
                marker['target_dir'] = max(matches, key=lambda pair: len(pair[0].parts))[1]
        print('prebuild: ' + subprocess.list2cmdline(command), flush=True)
        with log_path.open('w', encoding='utf-8') as log:
            log.write(f'Command: {json.dumps(command)}\nWorking directory: {cwd}\nHEAD: {head}\n'
                      f'Git status --short empty: {clean}\nStart: {start}\n\n')
            log.write(json.dumps(marker) + '\n')
            log.flush()
            try:
                result = subprocess.run(command, cwd=cwd, env=self.environment, stdout=subprocess.PIPE,
                                        stderr=subprocess.STDOUT if ordered else subprocess.PIPE,
                                        text=True, encoding='utf-8', errors='replace')
                result.stderr = result.stderr or ''
            except OSError as error:
                result = subprocess.CompletedProcess(command, 127, '', str(error))
            log.write(result.stdout)
            log.write('\n' + result.stderr)
            log.write(f'\nEnd: {datetime.now(timezone.utc).isoformat()}\nExit code: {result.returncode}\n')
        retained = log_path.resolve()
        target = self.repository / 'target'
        evidence = target / 'lane-evidence'
        if retained.is_relative_to(target):
            evidence.mkdir(parents=True, exist_ok=True)
            retained_name = retention_name(retained.relative_to(target), head, start)
            retained = evidence / retained_name
            shutil.copyfile(log_path, retained)
        self.logs.append(retained)
        capture_record(self.environment, retained)
        if command[:2] == ['cargo', 'metadata'] and result.returncode == 0:
            metadata = json.loads(result.stdout)
            if 'workspace_root' in metadata and 'target_directory' in metadata:
                self.target_directories[Path(metadata['workspace_root']).resolve()] = metadata['target_directory']
        if result.stderr:
            print(result.stderr[-6000:], end='', flush=True)
        return result
