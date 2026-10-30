"""Prebuild and retain Store's independently executed process prerequisites."""
import hashlib
import json
from pathlib import Path
import shutil
import sys
from urllib.parse import unquote

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from filesystem_path import filesystem_path

from logs import json_objects
from native_features import load as load_native_features, validate as validate_native_libraries, NativeFeatureRefusal


REQUESTS = [
    {'package': 'worth-store-offline-integrity-observer',
     'binary': 'physical_store_integrity_observer', 'features': []},
    {'package': 'worth-store-offline-verifier',
     'binary': 'physical_store_offline_observer', 'features': []},
    {'package': 'worth-store', 'binary': 'physical_store_c8_writer',
     'features': ['worth-store/volatile-sync-for-tests']},
    {'package': 'worth-store-recovery-runtime', 'binary': 'physical_store_recover',
     'features': ['worth-store-recovery-runtime/certification-test-authority',
                  'worth-store/volatile-sync-for-tests']},
]


class ExecutablePrebuildFailure(ValueError):
    pass


def retain(events, directory, requests=REQUESTS):
    directory = Path(directory).resolve()
    filesystem_path(directory).mkdir(parents=True, exist_ok=True)
    executables = []
    for request in requests:
        artifacts = [event for event in events if event.get('reason') == 'compiler-artifact'
                     and event['target']['name'] == request['binary']
                     and event['target']['kind'] == ['bin'] and not event['profile']['test']
                     and event.get('executable')]
        if len(artifacts) != 1:
            raise ExecutablePrebuildFailure(f'expected one independently built executable: {request}')
        artifact = artifacts[0]
        package_id = artifact['package_id']
        location, _, fragment = package_id.partition('#')
        package_name = fragment.split('@')[0] if '@' in fragment else unquote(location.rstrip('/').rsplit('/', 1)[-1])
        if request['package'] != package_name:
            raise ExecutablePrebuildFailure(f'executable belongs to another package: {package_id}')
        source = Path(artifact['executable'])
        digest = hashlib.sha256(filesystem_path(source).read_bytes()).hexdigest()
        destination = directory / f'{request["binary"]}-{digest}{source.suffix}'
        if not filesystem_path(destination).exists():
            shutil.copy2(filesystem_path(source), filesystem_path(destination))
        executables.append(dict(request, path=str(destination),
                                built_features=artifact['features'], profile=artifact['profile']))
    manifest = directory / 'executables.json'
    filesystem_path(manifest).write_text(json.dumps({'schema': 1, 'executables': executables}, indent=2) + '\n', encoding='utf-8')
    return {'WORTH_PREBUILT_EXECUTABLES': str(manifest)}


def require_native(repository, directory):
    native = load_native_features(directory, Path(repository) / 'workspaces/worth-store')
    required = {feature for request in REQUESTS for feature in request['features']}
    missing = required - set(native.features)
    if missing:
        raise NativeFeatureRefusal('native graph omits required executable features: ' + ', '.join(sorted(missing)))
    return native


def prepare(repository, output_directory, cargo_runner, profile='dev', *, native):
    workspace = Path(repository) / 'workspaces/worth-store'
    output = Path(output_directory)
    output.mkdir(parents=True, exist_ok=True)
    if native.workspace != workspace.resolve():
        raise NativeFeatureRefusal('independent executable graph belongs to another workspace')
    receipt = {'schema': 1, 'features': native.features, 'inputs': native.inputs}
    (output / 'native-feature-graph.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    result = cargo_runner(['cargo', 'build', '--workspace', '--bins', '--locked',
                           '--profile', profile, '--features',
                           ','.join(native.features),
                           '--manifest-path', str(workspace / 'Cargo.toml')],
                          workspace, output / 'executables-build.log')
    if result.returncode:
        raise ExecutablePrebuildFailure('Store executable prebuild failed; see executables-build.log')
    events = list(json_objects(result.stdout))
    receipt['validated_shared_libraries'] = validate_native_libraries(events, native)
    (output / 'native-feature-graph.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    return retain(events, output / 'executables')
