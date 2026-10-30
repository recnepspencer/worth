"""Bind measured IR to Cargo's exact source and hashed linked artifact."""
import hashlib
import os
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from filesystem_path import filesystem_path
from logs import json_objects


LIBRARY_KINDS = {'lib', 'rlib', 'dylib', 'cdylib', 'staticlib', 'proc-macro'}


def path_identity(value):
    value = str(value)
    if value.startswith('\\\\?\\UNC\\'):
        value = '\\\\' + value[8:]
    elif value.startswith('\\\\?\\'):
        value = value[4:]
    return os.path.normcase(str(Path(value).resolve()))


def ir_file(event):
    """Use the hash Cargo actually linked; never search a directory for IR."""
    target = event['target']
    library = not event['profile']['test'] and set(target['kind']) <= LIBRARY_KINDS
    crate = target['name'].replace('-', '_')
    pattern = re.compile(r'(?:lib)?' + re.escape(crate) + r'-[0-9a-f]+$' if library
                         else re.escape(crate) + r'-[0-9a-f]+$')
    candidates, linked = set(), False
    filenames = event.get('filenames')
    if not isinstance(filenames, list) or not filenames:
        raise ValueError('measured Cargo artifact has no filenames')
    executable = event.get('executable')
    for filename in filenames:
        path = Path(filename)
        if not path.is_absolute():
            raise ValueError('Cargo artifact filename must be absolute')
        if path.parent.name != 'deps':
            continue
        suffix = path.suffix
        if library:
            if suffix not in ('.rlib', '.rmeta', '.dll', '.so', '.dylib', '.a', '.lib'):
                continue
            stem = path.stem
            if not pattern.fullmatch(stem):
                continue
            # Rust's IR filename omits only the library filename's `lib` prefix.
            if stem.startswith('lib' + crate + '-'):
                stem = stem[3:]
            is_linked = suffix != '.rmeta'
        else:
            if executable is None or path_identity(path) != path_identity(executable):
                continue
            stem = path.stem if suffix == '.exe' else path.name
            if not pattern.fullmatch(stem):
                continue
            is_linked = True
        if is_linked:
            if not filesystem_path(path).is_file():
                raise ValueError(f'Cargo linked artifact is missing: {path}')
            linked = True
        candidates.add(path.parent / (stem + '.ll'))
    if not linked or len(candidates) != 1:
        raise ValueError('measured Cargo artifact has no unique hashed linked output')
    path = candidates.pop()
    if not filesystem_path(path).is_file():
        raise ValueError(f'Cargo artifact has no exact default LLVM IR file: {path}')
    data = filesystem_path(path).read_bytes()
    if not data:
        raise ValueError(f'Cargo artifact LLVM IR file is empty: {path}')
    return {'path': str(path), 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}


def attributed(text, wanted):
    """Require the declared normal units and retain additional library harnesses."""
    by_manifest = {}
    for row in wanted:
        key = (path_identity(row['manifest']), path_identity(row['source']), row['cargo_target'])
        if key in by_manifest:
            raise ValueError(f'duplicate LLVM target declaration: {key}')
        by_manifest[key] = row
    observed = {}
    for event in json_objects(text):
        if event.get('reason') != 'compiler-artifact':
            continue
        target = event.get('target', {})
        key = (path_identity(event.get('manifest_path', '')),
               path_identity(target.get('src_path', '')), target.get('name'))
        row = by_manifest.get(key)
        if row is None:
            continue
        kinds, profile = target.get('kind'), event.get('profile')
        valid_kind = (kinds == ['test'] if row['family'] != 'lib'
                      else isinstance(kinds, list) and bool(kinds) and set(kinds) <= LIBRARY_KINDS)
        if not valid_kind:
            raise ValueError(f'wrong Cargo target kind for {row["package"]}/{row["target"]}')
        if not isinstance(profile, dict) or not isinstance(profile.get('test'), bool):
            raise ValueError('measured Cargo artifact has no actual test profile')
        if not isinstance(event.get('package_id'), str) or not event['package_id']:
            raise ValueError('measured Cargo artifact has no package identity')
        features = event.get('features')
        if not isinstance(features, list) or any(not isinstance(value, str) for value in features):
            raise ValueError('measured Cargo artifact has no actual feature set')
        if not isinstance(event.get('fresh'), bool):
            raise ValueError('measured Cargo artifact has no cache provenance')
        identity = unit_identity(row)
        if row['family'] == 'lib' and profile['test']:
            identity += '-test-harness'
        if identity in observed:
            raise ValueError(f'duplicate measured Cargo unit: {identity}')
        observed[identity] = {'package': row['package'], 'package_id': event['package_id'],
                              'target': target, 'manifest': event['manifest_path'],
                              'profile': profile, 'features': features, 'fresh': event['fresh'],
                              'filenames': event['filenames'], 'executable': event.get('executable'),
                              'ir': ir_file(event)}
    missing = [unit_identity(row) for row in wanted if unit_identity(row) not in observed]
    if missing:
        raise ValueError('Cargo did not emit required LLVM units: ' + ', '.join(missing))
    return observed


def unit_identity(row):
    target = 'test-lib' if row['family'] == 'test' and row['target'] == 'lib' else row['target']
    return row['package'] + '-' + target
