"""Prebuild Proof's Cargo package-band and release-IR observations."""
import json
from pathlib import Path
import shutil
from urllib.parse import unquote


FIXTURES = ('crates/worth-court-proof/tests/worth_court_proof/sources/worth_proof/'
            'fixtures/band_guard_cargo_contract')
LEGAL_PACKAGES = ('worth-entry-consumer', 'worthy-entry-consumer', 'worth-cert-consumer')
DENIED_PACKAGES = ('worth-schema-consumer', 'worth-entry-cert-consumer', 'empty-prefix-consumer')
SPECIMENS = {'baseline': 'zero-cost-baseline', 'guarded': 'zero-cost-guarded'}


class ProofPrebuildError(RuntimeError):
    """The runner could not establish a required raw Proof observation."""


def prepare(repository, output_directory, cargo_runner):
    """Build prerequisites once; tests consume the observations, not verdicts.

    cargo_runner(command, cwd, log_path) returns stdout/stderr/returncode even
    for the intentional Cargo check failure. It owns complete lane capture.
    """
    repository, output = Path(repository).resolve(), Path(output_directory).resolve()
    output.mkdir(parents=True, exist_ok=True)
    ready = output / 'ready'
    ready.unlink(missing_ok=True)
    fixture_manifest = repository / FIXTURES / 'Cargo.toml'
    if not fixture_manifest.is_file():
        raise ProofPrebuildError(f'moved Proof fixture manifest is missing: {fixture_manifest}')
    command = ['cargo', 'check', '--workspace', '--keep-going', '--offline', '--locked',
               '--message-format=json', '--manifest-path', str(fixture_manifest),
               '--target-dir', str(repository / 'target')]
    checked = cargo_runner(command, repository, output / 'package-check.log')
    package_observations = checked_packages(checked)
    for name, observation in package_observations.items():
        (output / f'{name}.outcome').write_text(observation['outcome'] + '\n', encoding='utf-8')
        (output / f'{name}.stdout').write_text(observation['stdout'], encoding='utf-8')
        (output / f'{name}.stderr').write_text(observation['stderr'], encoding='utf-8')

    tree_command = ['cargo', 'tree', '--package', 'worth-proof', '--edges', 'normal',
                    '--depth', '1', '--prefix', 'none', '--offline', '--locked',
                    '--manifest-path', str(repository / 'crates/worth-proof/Cargo.toml')]
    tree = cargo_runner(tree_command, repository, output / 'normal-dependencies.log')
    if tree.returncode:
        raise ProofPrebuildError(f'Proof normal dependency observation failed:\n{tree.stderr}')
    write_observation(output, 'normal-dependencies', tree.returncode, text(tree.stdout), text(tree.stderr))

    specimen_manifest = write_specimens(repository, output)
    for label, package in SPECIMENS.items():
        ir_path = output / f'{label}.ll'
        ir_path.unlink(missing_ok=True)
        command = ['cargo', 'rustc', '--release', '--lib', '--offline', '--message-format=json',
                   '--manifest-path', str(specimen_manifest), '--package', package,
                   '--target-dir', str(repository / 'target'), '--', f'--emit=llvm-ir={ir_path}']
        compiled = cargo_runner(command, repository, output / f'{label}-ir.log')
        if compiled.returncode or not ir_path.is_file():
            raise ProofPrebuildError(f'{label} release IR was not produced:\n{compiled.stderr}')
        write_observation(output, f'{label}-ir', compiled.returncode,
                          text(compiled.stdout), text(compiled.stderr))
    ready.write_text('1\n', encoding='utf-8')
    return output


def text(value):
    return value.decode('utf-8', errors='replace') if isinstance(value, bytes) else value


def package_name(package_id):
    stem, _, fragment = package_id.rpartition('#')
    if '@' in fragment:
        return fragment.split('@', 1)[0]
    if fragment and not fragment[0].isdigit():
        return fragment
    return unquote(stem.rsplit('/', 1)[-1])


def checked_packages(completed):
    """Attribute a batched Cargo check by actual package diagnostics/artifacts."""
    wanted = set(LEGAL_PACKAGES + DENIED_PACKAGES)
    events = {name: [] for name in wanted}
    artifacts, errors = set(), set()
    rendered = {name: [] for name in wanted}
    for line in text(completed.stdout).splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        name = package_name(event.get('package_id', ''))
        if name not in wanted:
            continue
        events[name].append(line)
        if event.get('reason') == 'compiler-artifact':
            artifacts.add(name)
        if event.get('reason') == 'compiler-message' and event['message']['level'] == 'error':
            errors.add(name)
            rendered[name].append(event['message'].get('rendered') or event['message']['message'])
    missing = wanted - (artifacts | errors)
    if missing:
        raise ProofPrebuildError(f'Cargo produced no package observation for {sorted(missing)}; '
                                 f'exit={completed.returncode}\n{text(completed.stderr)}')
    if completed.returncode and not errors:
        raise ProofPrebuildError(f'Cargo check failed without attributed compiler errors:\n{completed.stderr}')
    return {name: {'outcome': 'denied' if name in errors else 'compiled',
                   'stdout': '\n'.join(events[name]) + '\n',
                   'stderr': '\n'.join(rendered[name])}
            for name in sorted(wanted)}


def write_observation(output, name, exit_code, stdout, stderr):
    (output / f'{name}.exit').write_text(f'{exit_code}\n', encoding='utf-8')
    (output / f'{name}.stdout').write_text(stdout, encoding='utf-8')
    (output / f'{name}.stderr').write_text(stderr, encoding='utf-8')


def write_specimens(repository, output):
    """Give both source variants distinct package identities and one target dir."""
    workspace = output / 'runtime-specimens'
    workspace.mkdir(parents=True, exist_ok=True)
    manifest = workspace / 'Cargo.toml'
    manifest.write_text('[workspace]\nresolver = "2"\nmembers = ["baseline", "guarded"]\n',
                        encoding='utf-8')
    # Preserve the repository's locked dependency versions for path resolution.
    shutil.copyfile(repository / 'Cargo.lock', workspace / 'Cargo.lock')
    proof = (repository / 'crates/worth-proof').as_posix()
    for label, package in SPECIMENS.items():
        root = workspace / label
        (root / 'src').mkdir(parents=True, exist_ok=True)
        (root / 'Cargo.toml').write_text(
            f'[package]\nname = "{package}"\nversion = "0.1.0"\nedition = "2021"\n'
            f'publish = false\n\n[dependencies]\nworth-proof = {{ path = {json.dumps(proof)} }}\n',
            encoding='utf-8')
        guard = ('worth_proof::band_guard!("zero-cost-");\n' if label == 'guarded'
                 else 'use worth_proof as _;\n')
        (root / 'src/lib.rs').write_text(
            guard + '#[no_mangle]\npub extern "C" fn observed_runtime_surface(value: u32) -> u32 {\n'
                    '    value.wrapping_add(1)\n}\n', encoding='utf-8')
    return manifest
