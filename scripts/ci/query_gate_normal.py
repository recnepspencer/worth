"""Compile committed Root input ownership into a bounded affected normal plan."""
from pathlib import PurePosixPath
import re
import subprocess
import tomllib

from query_gate_courts import COURT_OWNERS, TEST_SUITES, CATEGORY_SUITES, landing_suites


# Closed self-adoption contract, not a generic exemption for scripts or docs.
TOOLING = {
    'scripts/ci/query_gate.py', 'scripts/ci/query_gate_selection.py',
    'scripts/ci/query_gate_cost.py', 'scripts/ci/query_gate_normal.py',
    'scripts/ci/query_gate_cli.py', 'scripts/test_cost/doc_contracts.py',
    'scripts/test_cost/run_lane.py', 'scripts/test_cost/landing_plan.py',
    'scripts/prepare-constitution-hook.ps1',
}
TESTS = {
    'scripts/ci/test_query_gate_normal.py',
    'scripts/ci/test_query_gate_normal_protocol.py',
    'scripts/ci/test_query_gate_normal_selection.py', 'scripts/ci/test_query_gate_selection.py',
    'scripts/ci/test_query_gate_cost.py', 'scripts/ci/test_query_gate_documentation.py',
    'scripts/ci/test_query_gate_environment.py', 'scripts/ci/test_query_gate_capture.py',
    'scripts/test_cost/test_doc_contracts.py', 'scripts/test_cost/test_landing_plan.py',
    'scripts/test_cost/test_run_lane.py', 'scripts/test_cost/test_doc_lane.py',
    'scripts/test_cost/test_unit_plan.py', 'scripts/test_cost/test_nextest_build.py',
}
GUIDE = 'docs/normal-gate.md'


def git(root, *args):
    return subprocess.check_output(['git', *args], cwd=root).decode().strip()


def source_state(root):
    if git(root, 'status', '--porcelain=v1', '--untracked-files=all'):
        raise ValueError('normal requires a clean committed source, including untracked inputs')
    hidden = [line for line in git(root, 'ls-files', '-v').splitlines()
              if line[0].islower() or line[0] == 'S']
    if hidden:
        raise ValueError('normal refuses hidden tracked inputs')
    return dict(commit=git(root, 'rev-parse', 'HEAD'), tree=git(root, 'rev-parse', 'HEAD^{tree}'))


def inventory(root, commit):
    files = {}
    raw = subprocess.check_output(['git', 'ls-tree', '-rz', commit], cwd=root)
    for row in raw.split(b'\0'):
        if row:
            description, path = row.split(b'\t', 1)
            files[path.decode()] = description.split()[0].decode()
    return files


def document(root, commit, path):
    return tomllib.loads(git(root, 'show', commit + ':' + path))


def root_members(root, commit, files):
    configuration = document(root, commit, 'Cargo.toml')['workspace']
    result = {}
    for directory in configuration['members']:
        if (not isinstance(directory, str) or not directory or
                PurePosixPath(directory).is_absolute() or
                any(p in ('', '.', '..') for p in directory.split('/')) or
                any(c in directory for c in '*?[]\\')):
            raise ValueError('unsupported Root member path declaration')
        path = directory + '/Cargo.toml'
        if files.get(path) != '100644':
            raise ValueError('Root member manifest missing or unsupported mode: ' + path)
        manifest = document(root, commit, path)
        name = manifest['package']['name']
        if name in result:
            raise ValueError('ambiguous Root package identity: ' + name)
        result[name] = dict(path=directory, manifest=manifest)
    return result


def owner(path, members, files):
    for directory in PurePosixPath(path).parents:
        manifest = directory.as_posix() + '/Cargo.toml'
        if manifest not in files:
            continue
        names = [name for name, row in members.items() if row['path'] == directory.as_posix()]
        if len(names) != 1:
            raise ValueError('nested or foreign source manifest: ' + path)
        relative = PurePosixPath(path).relative_to(directory)
        if relative.parts[0] not in ('src', 'tests'):
            raise ValueError('unsupported Root Rust input area: ' + path)
        return names[0]
    raise ValueError('unowned Root source: ' + path)


def required_packages(name, members):
    if name in COURT_OWNERS:
        return {name}  # Court edit retains its dependencies without running unrelated units.
    leaves = {court for court, owners in COURT_OWNERS.items() if name in owners}
    if not leaves:
        raise ValueError('Root source has no declared forward court ownership: ' + name)
    if not leaves <= members.keys():
        raise ValueError('declared forward court is not a Root member')
    return {name, *leaves}


def suites_for(packages):
    return [suite for suite in landing_suites('root') if suite in CATEGORY_SUITES or
            any(TEST_SUITES[suite][1]({'package': name, 'lib': True}) for name in packages)]


def compile_plan(root, base):
    if not isinstance(base, str) or not re.fullmatch('[0-9a-f]{40}', base):
        raise ValueError('normal base must be a full committed object identity')
    state = source_state(root)
    if git(root, 'rev-parse', base + '^{commit}') != base:
        raise ValueError('normal base is not the specified commit')
    subprocess.run(['git', 'merge-base', '--is-ancestor', base, state['commit']],
                   cwd=root, check=True, capture_output=True)
    fetched = git(root, 'rev-parse', 'refs/remotes/origin/master')
    if fetched != base:
        raise ValueError('normal base differs from fetched master; integrate by meaning first')
    paths = git(root, 'diff', '--no-renames', '--name-only', base, state['commit']).splitlines()
    if not paths:
        raise ValueError('normal requires changed inputs')
    sides = [(inventory(root, ref), ref) for ref in (base, state['commit'])]
    members = [root_members(root, ref, files) for files, ref in sides]
    packages, classified, tools = set(), [], False
    for path in paths:
        for files, _ in sides:
            if path in files and files[path] != '100644':
                raise ValueError('unsupported changed input mode: ' + path)
        if path in TOOLING or path in TESTS:
            tools = True
            classified.append(dict(path=path, kind='tooling'))
        elif path == GUIDE:
            classified.append(dict(path=path, kind='guide'))
        elif path.endswith('.rs'):
            owned = set()
            for (files, _), definitions in zip(sides, members):
                if path in files:
                    name = owner(path, definitions, files)
                    owned.add(name)
                    packages.update(required_packages(name, definitions))
            classified.append(dict(path=path, kind='rust', owners=sorted(owned)))
        else:
            raise ValueError('unsupported normal input: ' + path)
    if GUIDE in paths and not set(paths) & TOOLING:
        raise ValueError('normal guide requires its actual provider change')
    if not packages and not tools:
        raise ValueError('normal request has no supported owner')
    if not packages <= members[1].keys():
        raise ValueError('required current Root owner/court missing')
    docs = []
    for name in sorted(packages):
        row = members[1][name]
        library = row['manifest'].get('lib', {})
        path = row['path'] + '/' + library.get('path', 'src/lib.rs')
        if path in sides[1][0] and ('lib' in row['manifest'] or
                                  row['manifest']['package'].get('autolib', True)):
            docs.append(name)
    suites = suites_for(packages) if packages else []
    if docs:
        suites.append('root-doc')
    return dict(schema='worth.affected-normal.v1', source=state, base=base,
                mode='affected-root' if packages else 'source-only', paths=classified,
                packages=sorted(packages), unselected_packages=sorted(members[1].keys() - packages),
                doc_packages=docs, suites=suites,
                tooling=tools, required_courts=sorted(packages & COURT_OWNERS.keys()))


def assert_unchanged(root, plan):
    if source_state(root) != plan['source'] or git(root, 'rev-parse', 'refs/remotes/origin/master') != plan['base']:
        raise ValueError('normal source or fetched base moved during the run')


def command_plan(plan, driver):
    """Use existing command owners; normal never bootstraps checker tools."""
    checks = [*driver.COMMAND_SUITES['fmt'],
              *(row for row in driver.COMMAND_SUITES['boundary'] if row[0] != 'prepare-constitution')]
    commands = {'source-checks': checks}
    if plan['tooling']:
        commands['normal-tooling'] = [
            ('protocol-' + PurePosixPath(path).stem,
             [driver.sys.executable, '-m', 'unittest', 'discover', '-s', str(PurePosixPath(path).parent),
              '-p', PurePosixPath(path).name]) for path in sorted(TESTS)]
    if plan['packages']:
        commands['normal-clippy'] = [('clippy-affected', driver.clippy(
            '--locked', '--offline', '--profile', 'dev', *driver.packages(*plan['packages'])))]
    return commands
