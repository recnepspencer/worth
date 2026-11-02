"""Retain Cargo's actual dependency features when artifact JSON is ambiguous."""
import json
from pathlib import Path

from compiler_units import CompilerUnitRefusal, attribute


def dependency_features(event):
    # Filenames locate Cargo's own fingerprint; their hashes never identify a
    # declared unit. Only the recorded dependency graph refines attribution.
    artifact = next(Path(name) for name in event['filenames'] if name.endswith('.rlib'))
    fingerprints = artifact.parent.parent / '.fingerprint'
    stem = artifact.stem.removeprefix('lib')
    record = json.loads((fingerprints / stem / ('lib-' + event['target']['name'] + '.json')).read_text())
    observed = []
    for _, name, _, fingerprint in record['deps']:
        features = set()
        for candidate in fingerprints.glob('*/lib-' + name):
            if int.from_bytes(bytes.fromhex(candidate.read_text().strip()), 'little') == fingerprint:
                value = json.loads(candidate.with_suffix('.json').read_text())
                features.add(tuple(sorted(json.loads(value['features']))))
        if len(features) != 1:
            raise ValueError('Cargo dependency fingerprint is unavailable or ambiguous: ' + name)
        observed.append([name, list(next(iter(features)))])
    return observed


def retain_dependencies(text, declaration):
    lines = []
    for line in text.splitlines(keepends=True):
        try:
            event = json.loads(line)
        except ValueError:
            lines.append(line)
            continue
        if isinstance(event, dict) and event.get('reason') == 'compiler-artifact':
            try:
                attribute(event, declaration)
            except CompilerUnitRefusal:
                try:
                    event['cargo_dependency_features'] = dependency_features(event)
                except (OSError, ValueError, KeyError, StopIteration):
                    pass  # Missing evidence still refuses at the ordinary attribution boundary.
                else:
                    line = json.dumps(event) + '\n'
        lines.append(line)
    return ''.join(lines)
