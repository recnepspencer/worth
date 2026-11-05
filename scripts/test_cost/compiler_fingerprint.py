"""Resolve otherwise indistinguishable units through Cargo's actual dependency links.

Artifact suffixes locate observations; they never declare compiler identity.
Cargo's fingerprint dependency values select the exact child fingerprints, whose
features/dependency graph distinguish predeclared units. Missing or ambiguous
observations refuse rather than assigning another unit's compilation allowance.
"""
import json
from pathlib import Path


class FingerprintRefusal(ValueError):
    pass


def crate_name(row):
    return row['target']['name'].replace('-', '_')


def read(path):
    try:
        return json.loads(path.read_text(encoding='utf-8'))
    except (OSError, ValueError) as error:
        raise FingerprintRefusal(f'Cargo fingerprint unavailable: {path}') from error


def original(event):
    name = event['target']['name'].replace('-', '_')
    located = set()
    for filename in event.get('filenames', []):
        artifact = Path(filename)
        stem = artifact.stem.removeprefix('lib')
        prefix = name + '-'
        if not stem.startswith(prefix):
            continue
        suffix = stem[len(prefix):]
        root = artifact.parent.parent / '.fingerprint'
        located.update(root.glob(f'*-{suffix}/lib-{name}.json'))
    if len(located) != 1:
        raise FingerprintRefusal('artifact has no unique Cargo fingerprint observation')
    path = next(iter(located))
    return path.parent.parent, read(path)


def dependency(root, name, fingerprint):
    found = []
    for path in root.glob(f'*/lib-{name}.json'):
        try:
            recorded = int.from_bytes(bytes.fromhex(path.with_suffix('').read_text().strip()),
                                      'little')
        except (OSError, ValueError):
            continue
        if recorded == fingerprint:
            found.append(read(path))
    # Identical physical observations at two locations may be shared; differing
    # observations with the same fingerprint are not evidence of identity.
    unique = {json.dumps(value, sort_keys=True): value for value in found}
    if len(unique) != 1:
        raise FingerprintRefusal(f'Cargo dependency {name} has no unique fingerprint')
    return next(iter(unique.values()))


def resolve(event, candidates, declaration):
    root, observed = original(event)
    units = {row['identity']: row for row in declaration['units']}

    def select(rows, node, active):
        rows = list(rows)
        # A fingerprint dependency name is not a package/version identity.
        # Never let feature differences choose between identically named
        # packages; that requires another Cargo observation.
        if len({row['package'] for row in rows}) != 1:
            raise FingerprintRefusal('dependency name spans multiple Cargo packages')
        try:
            features = json.loads(node['features'])
            if not isinstance(features, list) or any(not isinstance(value, str) for value in features):
                raise ValueError('Cargo features must be a string list')
        except (KeyError, TypeError, ValueError) as error:
            raise FingerprintRefusal('Cargo fingerprint features are unavailable') from error
        features = sorted(features)
        rows = {row['identity']: row for row in rows if row['features'] == features}
        if len(rows) == 1:
            return next(iter(rows))
        if not rows or set(rows) <= active:
            raise FingerprintRefusal('Cargo dependency observations do not distinguish units')
        alternatives = dict(rows)
        try:
            observed_dependencies = {name: value for _, name, _, value in node['deps']}
        except (KeyError, TypeError, ValueError) as error:
            raise FingerprintRefusal('Cargo fingerprint dependencies are unavailable') from error
        names = {crate_name(units[key]) for row in alternatives.values()
                 for key in row['dependency_units'] if key in units}
        for name in sorted(names):
            choices = {key: units[key] for row in alternatives.values()
                       for key in row['dependency_units']
                       if key in units and crate_name(units[key]) == name}
            if len(choices) < 2:
                continue
            if name not in observed_dependencies:
                raise FingerprintRefusal(f'Cargo dependency observation missing: {name}')
            child = dependency(root, name, observed_dependencies[name])
            selected = select(choices.values(), child, active | set(rows))
            rows = {key: row for key, row in rows.items()
                    if selected in row['dependency_units']}
        if len(rows) == 1:
            return next(iter(rows))
        raise FingerprintRefusal('Cargo dependency graph remains ambiguous')

    return select(candidates, observed, set())
