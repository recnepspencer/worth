"""Declare Cargo's required graph without compiling or changing build inputs."""
import hashlib
import json
import os
from pathlib import Path
import tomllib
from urllib.parse import urlsplit
from urllib.request import url2pathname

from compiler_units import CompilerUnitRefusal, declarations, digest


def command_for(command):
    if command[:1] != ['cargo'] or '--unit-graph' in command:
        return None
    subcommand = command[1]
    if subcommand == 'nextest':
        if any(flag == '--binaries-metadata' or flag.startswith('--binaries-metadata=') for flag in command):
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
    # --timings adds a build-started JSON record before Cargo's unit graph.
    # This subprocess only declares units; retain telemetry on the real build.
    result = [argument for index, argument in enumerate(result)
              if index >= boundary or not (argument == '--timings' or argument.startswith('--timings='))]
    boundary = result.index('--') if '--' in result else len(result)
    result[boundary:boundary] = ['--unit-graph', '-Z', 'unstable-options']
    return result


def config_inputs(cwd, environment, configurations):
    roots = list(Path(cwd).resolve().parents)
    roots.insert(0, Path(cwd).resolve())
    home = Path(environment.get('CARGO_HOME', Path.home() / '.cargo'))
    paths = [home / name for name in ('config', 'config.toml')]
    paths += [root / '.cargo' / name for root in roots for name in ('config', 'config.toml')]
    paths += [(Path(cwd)/value).resolve() for value in configurations if '=' not in value]
    return {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(set(paths)) if path.is_file()}


def manifest_paths(command, cwd, graph):
    roots = set()
    for row in graph['units']:
        package = row['pkg_id']
        if package.startswith('path+file:'):
            parsed = urlsplit(package.removeprefix('path+'))
            path = url2pathname(parsed.path)
            if parsed.netloc:
                path = '//' + parsed.netloc + path
            roots.add(Path(path)/'Cargo.toml')
    for index, argument in enumerate(command):
        if argument == '--manifest-path':
            roots.add(Path(cwd)/command[index + 1])
        elif argument.startswith('--manifest-path='):
            roots.add(Path(cwd)/argument.split('=', 1)[1])
    if not roots:
        found = next((root/'Cargo.toml' for root in (Path(cwd), *Path(cwd).parents)
                      if (root/'Cargo.toml').is_file()), None)
        if found is not None:
            roots.add(found)
    paths = set()
    for manifest in roots:
        manifest = manifest.resolve()
        for directory in (manifest.parent, *manifest.parent.parents):
            path = directory/'Cargo.toml'
            if not path.is_file():
                continue
            paths.add(path)
            if 'workspace' in tomllib.loads(path.read_text(encoding='utf-8')):
                paths.add(directory/'Cargo.lock')
                break
        else:
            paths.add(manifest.with_name('Cargo.lock'))
    return sorted(paths)


def inputs(command, cwd, environment, manifests=()):
    environment = os.environ if environment is None else environment
    compiler_environment = {key: value for key, value in environment.items()
                            if key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS',
                                       'CARGO_ENCODED_RUSTDOCFLAGS', 'RUSTC', 'RUSTDOC', 'RUSTC_WRAPPER',
                                       'RUSTC_WORKSPACE_WRAPPER', 'RUSTC_BOOTSTRAP', 'CARGO_INCREMENTAL',
                                       'RUSTUP_TOOLCHAIN', 'RUSTUP_HOME',
                                       'WORTH_IMPLEMENTATION_BUILD_ID')
                            or key.startswith(('CARGO_PROFILE_', 'CARGO_TARGET_', 'CARGO_BUILD_'))}
    flags = []
    for index, argument in enumerate(command):
        if argument == '--config':
            flags.append(command[index + 1])
        elif argument.startswith('--config='):
            flags.append(argument.split('=', 1)[1])
    configurations = list(flags)
    if command[1:2] == ['rustc'] and '--' in command:
        flags += command[command.index('--') + 1:]
    compiler_inputs = {'environment': compiler_environment,
                       'config_files': config_inputs(cwd, environment, configurations),
                       'command_flags': flags}
    return {'command': list(command), 'cwd': str(Path(cwd).resolve()),
            'compiler_inputs': compiler_inputs,
            'manifests': {str(path): hashlib.sha256(Path(path).read_bytes()).hexdigest()
                          if Path(path).is_file() else None for path in manifests}}


def checked_test_roots(command, graph):
    if command[1] not in ('check', 'clippy'):
        return ()
    explicit_example = any(argument == '--example' or argument.startswith('--example=')
                           for argument in command)
    if '--tests' in command and not explicit_example and not any(flag in command for flag in
            ('--all-targets', '--lib', '--bins', '--examples', '--benches')):
        return graph['roots']
    if '--all-targets' not in command and '--tests' not in command:
        return ()
    # Cargo erases the harness bit in a check graph. A target's normal stage
    # can be a dependency; its checked test stage is a leaf root. Examples
    # marked test=true also have ordinary and cfg(test) checks, even when their
    # executable uses harness=false. Preserve both declared
    # stages, including an otherwise identical pair with no incoming edges.
    incoming = {edge['index'] for row in graph['units'] for edge in row['dependencies']}
    groups = {}
    for index, row in enumerate(graph['units']):
        kinds = set(row['target']['kind'])
        library = bool(kinds & {'lib', 'rlib', 'dylib', 'cdylib', 'staticlib', 'proc-macro'})
        if row['mode'] == 'check' and (library or row['target']['kind'] in (['bin'], ['example'])):
            key = digest({key: value for key, value in row.items() if key != 'dependencies'})
            groups.setdefault(key, []).append(index)
    harnesses = []
    cargo_arguments = command[:command.index('--')] if '--' in command else command
    only_test_binaries = '--tests' in cargo_arguments and not any(
        argument in ('--all-targets', '--bins', '--bin') or argument.startswith('--bin=')
        for argument in cargo_arguments)
    ordinary_examples = {
        argument.split('=', 1)[1] if argument.startswith('--example=') else cargo_arguments[index + 1]
        for index, argument in enumerate(cargo_arguments)
        if argument.startswith('--example=') or argument == '--example'
    }
    only_test_examples = '--tests' in cargo_arguments and not any(
        argument in ('--all-targets', '--examples') for argument in cargo_arguments)
    for indices in groups.values():
        if len(indices) == 1 and (only_test_binaries or only_test_examples):
            index = indices[0]
            row = graph['units'][index]
            # --tests includes test-enabled examples that were not explicitly
            # selected for an ordinary stage. Their harness is a singleton.
            test_selection = ((row['target']['kind'] == ['bin'] and only_test_binaries)
                or (row['target']['kind'] == ['example'] and only_test_examples
                    and row['target']['name'] not in ordinary_examples))
            if (test_selection and row['target'].get('test', False)
                    and index in graph['roots'] and index not in incoming):
                harnesses.append(index)
        if len(indices) != 2:
            continue
        leaves = [index for index in indices if index in graph['roots'] and index not in incoming]
        if len(leaves) == 1:
            harnesses.extend(leaves)
        elif len(leaves) == 2:
            first, second = (graph['units'][index]['dependencies'] for index in indices)
            if first == second:
                harnesses.append(leaves[1])
            elif all(edge in second for edge in first) and len(first) < len(second):
                harnesses.append(leaves[1])
            elif all(edge in first for edge in second) and len(second) < len(first):
                harnesses.append(leaves[0])
            else:
                raise CompilerUnitRefusal('checked target stages have no unique dependency/harness split')
    return harnesses


def declare(command, cwd, environment, observed):
    try:
        graph = json.loads(observed)
        test_roots = checked_test_roots(command, graph)
        units = declarations(graph, checked_test_roots=test_roots)
    except (KeyError, TypeError, ValueError) as error:
        raise CompilerUnitRefusal(f'could not declare required compiler units: {error}') from error
    contract = inputs(command, cwd, environment, manifest_paths(command, cwd, graph))
    shared = dict(contract['compiler_inputs'])
    # Incremental mode is already part of each declared Cargo profile. Comparing
    # the spelling of its override would invent drift for unaffected dependencies.
    primary_flags = command[command.index('--') + 1:] if command[1:2] == ['rustc'] and '--' in command else []
    configurations = shared['command_flags'][:-len(primary_flags)] if primary_flags else shared['command_flags']
    effective = []
    for value in configurations:
        try:
            parsed = tomllib.loads(value)
        except tomllib.TOMLDecodeError:
            parsed = None
        if parsed not in ({'build': {'incremental': True}}, {'build': {'incremental': False}}):
            effective.append(value)
        elif any(not isinstance(row['profile'].get('incremental'), bool) for row in units):
            raise CompilerUnitRefusal('Cargo did not declare the resolved incremental mode')
    shared['command_flags'] = effective
    rows = iter(units)
    for index, unit in enumerate(graph['units']):
        if unit['mode'] == 'run-custom-build':
            continue
        row = next(rows)
        # cargo rustc's trailing arguments apply only to the selected roots.
        row['compiler_inputs'] = digest(dict(shared, command_flags=effective +
            (primary_flags if index in graph['roots'] else [])))
    return {'schema': 1, 'type': 'compiler-unit-plan', 'contract': contract,
            'compiler_inputs': digest(contract['compiler_inputs']), 'units': units,
            'graph': graph}


def verify(plan, command, cwd, environment):
    if plan['contract'] != inputs(command, cwd, environment, plan['contract']['manifests']):
        raise CompilerUnitRefusal('compiler command, flags, environment, config or manifests drifted after declaration')
