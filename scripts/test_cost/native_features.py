"""Carry actual native compilation features into independent executable builds."""
from collections import defaultdict
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path

from logs import json_objects


class NativeFeatureRefusal(ValueError):
    pass


@dataclass(frozen=True)
class NativeFeatureGraph:
    workspace: Path
    features: tuple
    libraries: dict
    inputs: tuple


def library_identity(event):
    if event['profile']['test'] or event['target']['kind'] not in (
            ['lib'], ['rlib'], ['proc-macro'], ['dylib'], ['cdylib'], ['staticlib']):
        return None
    return (event['package_id'], event['target']['name'], tuple(event['target']['kind']))


def signature(event):
    return (tuple(sorted(event['features'])), json.dumps(event['profile'], sort_keys=True),
            tuple(sorted(event['filenames'])))


def load(directory, workspace):
    if directory is None:
        raise NativeFeatureRefusal('Store executable preparation requires the native build directory')
    directory, workspace = Path(directory), Path(workspace).resolve()
    paths = [directory / 'cargo-metadata.json', directory / 'binaries-build.log']
    try:
        metadata = json.loads(paths[0].read_text(encoding='utf-8'))
        if Path(metadata['workspace_root']).resolve() != workspace:
            raise NativeFeatureRefusal('native compilation belongs to another workspace')
        members = set(metadata['workspace_members'])
        packages = {row['id']: row for row in metadata['packages'] if row['id'] in members}
        if set(packages) != members:
            raise NativeFeatureRefusal('native workspace package inventory is incomplete')
        observed, libraries = defaultdict(set), defaultdict(set)
        events = list(json_objects(paths[1].read_text(encoding='utf-8')))
        count = 0
        for event in events:
            if event.get('reason') != 'compiler-artifact':
                continue
            if not isinstance(event['features'], list) or any(not isinstance(value, str) for value in event['features']):
                raise NativeFeatureRefusal('native artifact has malformed features')
            if event['target']['kind'] != ['custom-build']:
                count += 1
                if event['package_id'] in members:
                    observed[event['package_id']].add(tuple(sorted(event['features'])))
            identity = library_identity(event)
            if identity:
                libraries[identity].add(signature(event))
        if not count or not libraries:
            raise NativeFeatureRefusal('native compilation has no compiler artifact observations')
        features = []
        for package_id, variants in sorted(observed.items()):
            if len(variants) != 1:
                raise NativeFeatureRefusal('native package has multiple feature variants: ' + package_id)
            package = packages[package_id]
            for feature in next(iter(variants)):
                if feature not in package['features']:
                    raise NativeFeatureRefusal('native artifact feature is not declared: ' + feature)
                features.append(package['name'] + '/' + feature)
        inputs = tuple({'path': str(path.resolve()), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
                       for path in paths)
        return NativeFeatureGraph(workspace, tuple(sorted(features)), dict(libraries), inputs)
    except (OSError, KeyError, TypeError, ValueError) as error:
        if isinstance(error, NativeFeatureRefusal):
            raise
        raise NativeFeatureRefusal(f'cannot read native feature graph: {error}') from error


def validate(events, native):
    """Refuse a second library graph even when both variants are already cached."""
    compared = 0
    for event in events:
        if event.get('reason') != 'compiler-artifact':
            continue
        try:
            identity = library_identity(event)
            if identity in native.libraries:
                compared += 1
                if signature(event) not in native.libraries[identity]:
                    raise NativeFeatureRefusal('executable library graph differs from native compilation: ' + identity[0])
        except (KeyError, TypeError, ValueError) as error:
            if isinstance(error, NativeFeatureRefusal):
                raise
            raise NativeFeatureRefusal(f'malformed executable compiler observation: {error}') from error
    if not compared:
        raise NativeFeatureRefusal('executable compilation shares no observed native libraries')
    return compared
