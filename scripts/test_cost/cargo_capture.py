"""The runner's single Cargo process boundary and compiler-JSON capture."""
from datetime import datetime, timezone
import json
import hashlib
import re
from pathlib import Path
import subprocess
import shutil
import sys

from compiler_units import CompilerUnitRefusal, attribute as attribute_unit
from unit_plan import command_for, declare, verify


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
        if (not any(a == '--binaries-metadata' or a.startswith('--binaries-metadata=') for a in command)
                and not any(a.startswith('--cargo-message-format') for a in command)):
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


def compiler_profile(command):
    if command[:1] != ['cargo']:
        return None
    flags = ('--cargo-profile',) if command[1:2] == ['nextest'] else ('--profile',)
    boundary = command.index('--') if '--' in command else len(command)
    for index, argument in enumerate(command[:boundary]):
        for flag in flags:
            if argument == flag:
                return command[index + 1]
            if argument.startswith(flag + '='):
                return argument.split('=', 1)[1]
    if '--release' in command[:boundary]:
        return 'release'
    return 'test' if command[1:2] in (['test'], ['nextest']) else 'dev'


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

    def unit_plan(self, command, cwd, log_path):
        graph_command = command_for(command)
        if graph_command is None:
            return None
        import os
        original = self.environment
        self.environment = dict(os.environ if original is None else original)
        # Only Cargo's non-compiling declaration subprocess admits the unstable
        # graph output. The pinned compiler's real invocation never receives it.
        self.environment['RUSTC_BOOTSTRAP'] = '1'
        try:
            graph = self(graph_command, cwd, log_path.with_name(log_path.stem + '-unit-plan.log'))
        finally:
            self.environment = original
        if graph.returncode:
            raise CompilerUnitRefusal('Cargo could not declare required compiler units: ' + graph.stderr)
        plan = declare(command, cwd, self.environment, graph.stdout)
        plan['graph_log'] = str(self.logs[-1])
        log_path.with_name(log_path.stem + '-units.json').write_text(
            json.dumps(plan, indent=2) + '\n', encoding='utf-8')
        return plan

    def __call__(self, command, cwd, log_path):
        command = instrument(command)
        cwd, log_path = Path(cwd).resolve(), Path(log_path)
        log_path.parent.mkdir(parents=True, exist_ok=True)
        plan = self.unit_plan(command, cwd, log_path)
        head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.repository, text=True).strip()
        clean = not subprocess.check_output(['git', 'status', '--short'], cwd=self.repository, text=True).strip()
        start = datetime.now(timezone.utc).isoformat()
        ordered = command[:2] == ['cargo', 'test'] and '--doc' in command
        marker = {'type': 'cargo-invocation', 'command': command, 'cwd': str(cwd),
                  'capture': 'ordered' if ordered else 'separate',
                  'mode': 'check' if command[1:2] in (['check'], ['clippy']) else 'build',
                  'target_dir': None,
                  'cargo_profile': compiler_profile(command)}
        if plan is not None:
            marker['unit_plan'] = {key: plan[key] for key in ('units', 'compiler_inputs', 'graph_log')}
            marker['compiler_contract'] = plan['contract']
        if '--target-dir' in command:
            marker['target_dir'] = command[command.index('--target-dir') + 1]
        elif any(argument.startswith('--target-dir=') for argument in command):
            marker['target_dir'] = next(argument.split('=', 1)[1] for argument in command
                                        if argument.startswith('--target-dir='))
        else:
            matches = [(root, target) for root, target in self.target_directories.items()
                       if cwd.is_relative_to(root)]
            if matches:
                marker['target_dir'] = max(matches, key=lambda pair: len(pair[0].parts))[1]
        print('prebuild: ' + subprocess.list2cmdline(command), flush=True)
        refusal = None
        with log_path.open('w', encoding='utf-8') as log:
            log.write(f'Command: {json.dumps(command)}\nWorking directory: {cwd}\nHEAD: {head}\n'
                      f'Git status --short empty: {clean}\nStart: {start}\n\n')
            log.write(json.dumps(marker) + '\n')
            log.flush()
            try:
                if plan is not None:
                    verify(plan, command, cwd, self.environment)
                result = subprocess.run(command, cwd=cwd, env=self.environment, stdout=subprocess.PIPE,
                                        stderr=subprocess.STDOUT if ordered else subprocess.PIPE,
                                        text=True, encoding='utf-8', errors='replace')
                result.stderr = result.stderr or ''
            except CompilerUnitRefusal as error:
                refusal = error
                result = subprocess.CompletedProcess(command, 125, '', str(error))
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
        if refusal is not None:
            raise refusal
        if plan is not None:
            from logs import json_objects
            verify(plan, command, cwd, self.environment)
            for event in json_objects(result.stdout):
                if event.get('reason') == 'compiler-artifact':
                    attribute_unit(event, plan)
        if command[:2] == ['cargo', 'metadata'] and result.returncode == 0:
            metadata = json.loads(result.stdout)
            if 'workspace_root' in metadata and 'target_directory' in metadata:
                self.target_directories[Path(metadata['workspace_root']).resolve()] = metadata['target_directory']
        if result.stderr:
            print(result.stderr[-6000:], end='', flush=True)
        return result
