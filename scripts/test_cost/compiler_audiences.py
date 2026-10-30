"""Group fixture programs only when their complete direct-dependency surfaces agree."""
import copy
import hashlib
import json
import os
from pathlib import Path


def surface_key(dependencies, target_dependencies=None):
    value = copy.deepcopy({'dependencies': dependencies, 'target_dependencies': target_dependencies or {}})
    for table in [value['dependencies'], *value['target_dependencies'].values()]:
        for spec in table.values():
            if 'features' in spec:
                spec['features'] = sorted(set(spec['features']))
    return json.dumps(value, sort_keys=True, separators=(',', ':'))


def surface_identity(dependencies, target_dependencies=None):
    return hashlib.sha256(surface_key(dependencies, target_dependencies).encode()).hexdigest()


def source_identity(source):
    return os.path.normcase(str(Path(source).resolve()))


def group(records):
    packages, seen = {}, set()
    for record in records:
        surface = json.loads(surface_key(record['dependencies'], record.get('target_dependencies')))
        identity = surface_identity(surface['dependencies'], surface['target_dependencies'])
        package = packages.setdefault(identity, {
            'identity': identity, **surface,
            'sources': [], 'audiences': []})
        if record.get('audience') and record['audience'] not in package['audiences']:
            package['audiences'].append(record['audience'])
        for source in record['sources']:
            key = source_identity(source)
            if key in seen:
                raise ValueError(f'duplicate fixture source across compiler audiences: {source}')
            seen.add(key)
            package['sources'].append(Path(source).resolve())
    for package in packages.values():
        package['sources'].sort(key=source_identity)
        package['audiences'].sort()
    return [packages[key] for key in sorted(packages)]


def sources(environment):
    result, seen, identities = [], set(), set()
    for package in environment['packages']:
        identity = surface_identity(package['dependencies'], package.get('target_dependencies'))
        if identity != package['identity']:
            raise ValueError('compiler audience identity does not match its dependencies')
        if identity in identities:
            raise ValueError(f'duplicate compiler audience package: {identity}')
        identities.add(identity)
        for source in package['sources']:
            key = source_identity(source)
            if key in seen:
                raise ValueError(f'duplicate fixture source across compiler audiences: {source}')
            seen.add(key)
            result.append(Path(source).resolve())
    return result
