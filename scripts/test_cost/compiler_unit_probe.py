"""Bounded cold production/certification graph proof in Store's active target."""
import argparse
import json
from pathlib import Path
import shutil
import sys
import tomllib

from cargo_capture import CargoRunner
from compile_batch import verify_locked_dependencies
from compiler_features import violations as production_violations
from compiler_flags import arguments
from compiler_units import CompilerUnitRefusal
from logs import json_objects, read_lane
from ratchet import build_violations
from run_lane import child_environment


PROBE_FLAG = ['--cfg', 'worth_b3_cold_unit_probe']


def manifests(repository, output):
    repository, output = Path(repository).resolve(), Path(output).resolve()
    store = repository/'workspaces/worth-store/Cargo.toml'
    locked = tomllib.loads(store.with_name('Cargo.lock').read_text(encoding='utf-8'))
    sha2 = [row['version'] for row in locked['package'] if row['name'] == 'sha2']
    if len(sha2) != 1:
        raise CompilerUnitRefusal('bounded graph requires the exact single locked sha2 version')
    profiles = tomllib.loads(store.read_text(encoding='utf-8'))['profile']
    from manifest_edit import toml_value
    profile_text = '\n'.join('[profile.' + json.dumps(name) + ']\n' +
        '\n'.join(json.dumps(key) + '=' + toml_value(value) for key, value in row.items())
        for name, row in profiles.items())
    paths = {}
    for audience in ('production', 'certification'):
        directory = output/audience
        directory.mkdir(parents=True, exist_ok=True)
        proof = {'path': (repository/'crates/worth-proof').as_posix()}
        if audience == 'certification':
            proof['features'] = ['certification-test-authority']
        dependencies = {'worth-foundational': {'path': (repository/'crates/worth-foundational').as_posix()},
                        'worth-proof': proof, 'sha2': {'version': '=' + sha2[0]}}
        text = '[package]\nname="bounded-unit-' + audience + '"\nversion="0.0.0"\nedition="2024"\n'
        text += '[workspace]\nresolver="3"\n[dependencies]\n'
        text += '\n'.join(json.dumps(name) + '=' + toml_value(value) for name, value in dependencies.items())
        text += '\n[build-dependencies]\nsha2=' + json.dumps('=' + sha2[0]) + '\n' + profile_text + '\n'
        (directory/'Cargo.toml').write_text(text, encoding='utf-8')
        (directory/'src').mkdir(exist_ok=True)
        (directory/'src/main.rs').write_text('fn main() {}\n', encoding='utf-8')
        (directory/'build.rs').write_text('fn main() { let _ = sha2::Sha256::default(); }\n', encoding='utf-8')
        shutil.copyfile(store.with_name('Cargo.lock'), directory/'Cargo.lock')
        paths[audience] = directory/'Cargo.toml'
    return paths


def run(repository, output, runner):
    repository, output = Path(repository).resolve(), Path(output).resolve()
    store = repository/'workspaces/worth-store/Cargo.toml'
    paths = manifests(repository, output)
    artifacts, metadata = [], {}
    with arguments(runner, PROBE_FLAG) as flags:
        for audience, manifest in paths.items():
            resolved = runner(['cargo', 'metadata', '--offline', '--format-version', '1',
                               '--manifest-path', str(manifest)], store.parent, output/f'{audience}-metadata.log')
            if resolved.returncode:
                raise CompilerUnitRefusal('bounded audience metadata failed')
            metadata[audience] = json.loads(resolved.stdout)
            verify_locked_dependencies(store.with_name('Cargo.lock'), metadata[audience])
            if audience == 'production' and production_violations(store, metadata[audience]):
                raise CompilerUnitRefusal('bounded production graph enables certification features')
            command = ['cargo', 'check', '--bins', '--offline', '--locked', '--profile', 'diagnostic-test',
                       '--manifest-path', str(manifest), '--target-dir', str(store.parent/'target'),
                       '--config', 'build.incremental=false', *flags]
            observed = runner(command, store.parent, output/f'{audience}-check.log')
            if observed.returncode:
                raise CompilerUnitRefusal('bounded compiler graph failed: ' + audience)
            lane = read_lane(runner.logs[-1])
            artifacts.extend(lane['artifacts'])
    violations = build_violations({'build_observation': 'json', 'artifacts': artifacts},
                                  {'max_compiles_per_unit': 1, 'declared_cargo_profiles': ['diagnostic-test']})
    by_name = {}
    for artifact in artifacts:
        by_name.setdefault(artifact['target'], []).append(artifact)
    proof = by_name.get('worth_proof', [])
    foundation = by_name.get('worth_foundational', [])
    sha = by_name.get('sha2', [])
    if (len(proof) != 2 or len(foundation) != 2 or len(sha) != 2 or
            {row['unit']['output'] for row in sha} != {'check', 'codegen'} or
            len({row['unit']['identity'] for row in foundation}) != 2):
        raise CompilerUnitRefusal('cold capture did not observe both required graphs and host/check units')
    if violations:
        raise CompilerUnitRefusal('bounded graph exceeded unchanged budget: ' + json.dumps(violations))
    summary = {'scope': 'bounded-real-dependencies', 'cold_flag': PROBE_FLAG,
               'max_compiles_per_unit': 1, 'violations': violations,
               'compiled_units': len(artifacts), 'affected_shared_dependencies': {
                   name: rows for name, rows in by_name.items()
                   if name in ('worth_proof', 'worth_foundational', 'sha2')},
               'production_certification_features': {
                   name: [{'package': node['id'], 'features': node['features']}
                          for node in value['resolve']['nodes'] if 'worth-proof' in node['id']]
                   for name, value in metadata.items()},
               'logs': [str(path) for path in runner.logs]}
    (output/'result.json').write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    observed = run(args.root, args.output, CargoRunner(args.root, child_environment()))
    print(json.dumps({'compiled_units': observed['compiled_units'], 'violations': observed['violations']}))


if __name__ == '__main__':
    main()
