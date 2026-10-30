"""Declare Cargo's required graph without compiling or changing build inputs."""
import hashlib
import json
import os
from pathlib import Path

from compiler_units import CompilerUnitRefusal, declarations, digest


def command_for(command):
    if command[:1] != ['cargo'] or '--unit-graph' in command:
        return None
    subcommand = command[1]
    if subcommand == 'nextest':
        if '--binaries-metadata' in command:
            return None
        if command[2] != 'list':
            raise CompilerUnitRefusal('unplanned nextest compilation; prebuild through list')
        result = ['cargo', 'test', '--no-run']
        flags = iter(command[3:])
        for flag in flags:
            if flag in ('--message-format', '--list-type'):
                next(flags)
            elif flag.startswith(('--message-format=', '--cargo-message-format=', '--list-type=')):
                continue
            elif flag == '--cargo-profile':
                result += ['--profile', next(flags)]
            elif flag.startswith('--cargo-profile='):
                result.append('--profile=' + flag.split('=', 1)[1])
            else:
                result.append(flag)
    elif subcommand in ('build', 'check', 'test', 'rustc', 'clippy', 'run'):
        result = list(command)
        if subcommand == 'run':
            result[1] = 'build'
            result = result[:result.index('--')] if '--' in result else result
    else:
        return None
    boundary = result.index('--') if '--' in result else len(result)
    result[boundary:boundary] = ['--unit-graph', '-Z', 'unstable-options']
    return result


def config_inputs(cwd, environment):
    roots = list(Path(cwd).resolve().parents)
    roots.insert(0, Path(cwd).resolve())
    home = Path(environment.get('CARGO_HOME', Path.home() / '.cargo'))
    paths = [home / name for name in ('config', 'config.toml')]
    paths += [root / '.cargo' / name for root in roots for name in ('config', 'config.toml')]
    return {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(set(paths)) if path.is_file()}


def inputs(command, cwd, environment):
    environment = os.environ if environment is None else environment
    compiler_environment = {key: value for key, value in environment.items()
                            if key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                                       'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN', 'CARGO_BUILD_TARGET',
                                       'WORTH_IMPLEMENTATION_BUILD_ID')
                            or key.startswith(('CARGO_PROFILE_', 'CARGO_TARGET_'))}
    flags = []
    for index, argument in enumerate(command):
        if argument == '--config':
            flags.append(command[index + 1])
        elif argument.startswith('--config='):
            flags.append(argument.split('=', 1)[1])
    if command[1:2] == ['rustc'] and '--' in command:
        flags += command[command.index('--') + 1:]
    compiler_inputs = {'environment': compiler_environment, 'config_files': config_inputs(cwd, environment),
                       'command_flags': flags}
    return {'command': list(command), 'cwd': str(Path(cwd).resolve()),
            'compiler_inputs': compiler_inputs}


def declare(command, cwd, environment, observed):
    try:
        graph = json.loads(observed)
        units = declarations(graph)
    except (KeyError, TypeError, ValueError) as error:
        raise CompilerUnitRefusal(f'could not declare required compiler units: {error}') from error
    contract = inputs(command, cwd, environment)
    return {'schema': 1, 'type': 'compiler-unit-plan', 'contract': contract,
            'compiler_inputs': digest(contract['compiler_inputs']), 'units': units,
            'graph': graph}


def verify(plan, command, cwd, environment):
    if plan['contract'] != inputs(command, cwd, environment):
        raise CompilerUnitRefusal('compiler command, flags, environment or config drifted after declaration')
