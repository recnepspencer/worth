"""Bounded, hashed JSONL catalog trees for measured scheduling policy."""
import hashlib
from itertools import islice
import json
import os
from pathlib import Path


CAPACITY = 100
COLLECTIONS = {'sources', 'samples', 'rules', 'baselines', 'retired', 'pending'}


def digest(raw):
    return hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest()


def owned(directory, name):
    path = (Path(directory).resolve() / name).resolve()
    if path.parent != Path(directory).resolve():
        raise ValueError('policy catalog escapes its owner')
    return path


def index(directory):
    path = Path(directory) / 'index.json'
    if not path.exists():
        return {}
    result = json.loads(path.read_text(encoding='utf-8'))
    if result.get('schema') != 2 or set(result.get('catalogs', {})) != COLLECTIONS:
        raise ValueError('unsupported measured scheduling policy catalog schema')
    return result


def batches(rows):
    rows = iter(rows)
    while batch := list(islice(rows, CAPACITY)):
        yield batch


def write(directory, name, rows):
    directory = Path(directory).resolve()
    if not name or any(character not in 'abcdefghijklmnopqrstuvwxyz0123456789-_' for character in name):
        raise ValueError('invalid policy catalog name')
    directory.mkdir(parents=True, exist_ok=True)

    def emit(kind, values, level, ordinal):
        if any(not isinstance(value, dict) for value in values):
            raise ValueError('policy catalog rows must be JSON objects')
        text = json.dumps({'kind': kind, 'count': len(values)}, sort_keys=True) + '\n'
        text += ''.join(json.dumps(value, sort_keys=True) + '\n' for value in values)
        checksum = digest(text.encode('utf-8'))
        filename = f'{name}-{level:02}-{ordinal:06}-{checksum[:16]}.jsonl'
        path = owned(directory, filename)
        temporary = path.with_suffix('.tmp')
        temporary.write_text(text, encoding='utf-8')
        os.replace(temporary, path)
        count = len(values) if kind == 'leaf' else sum(value['records'] for value in values)
        return {'file': filename, 'sha256': checksum, 'records': count}

    current = [emit('leaf', batch, 0, ordinal) for ordinal, batch in enumerate(batches(rows))]
    if not current:
        current = [emit('leaf', [], 0, 0)]
    level = 1
    while len(current) > 1:
        current = [emit('node', batch, level, ordinal) for ordinal, batch in enumerate(batches(current))]
        level += 1
    return current[0]


def walk(directory, descriptor, ancestors, paths):
    if type(descriptor.get('records')) is not int or descriptor['records'] < 0:
        raise ValueError('policy catalog has an invalid record count')
    path = owned(directory, descriptor['file'])
    if path in ancestors:
        raise ValueError('policy catalog contains a cycle')
    raw = path.read_bytes()
    if digest(raw) != descriptor['sha256']:
        raise ValueError('policy catalog hash mismatch: ' + path.name)
    lines = raw.decode('utf-8').splitlines()
    if not lines or len(lines) > CAPACITY + 1:
        raise ValueError('policy catalog exceeds its bounded framing')
    header, rows = json.loads(lines[0]), [json.loads(line) for line in lines[1:]]
    if type(header.get('count')) is not int or header['count'] != len(rows):
        raise ValueError('policy catalog header contradicts its rows')
    if any(not isinstance(row, dict) for row in rows):
        raise ValueError('policy catalog contains a non-object row')
    paths.add(path.name)
    if header.get('kind') == 'leaf':
        if len(rows) != descriptor['records']:
            raise ValueError('policy catalog leaf count mismatch')
        yield from rows
    elif header.get('kind') == 'node':
        if not rows or sum(row['records'] for row in rows) != descriptor['records']:
            raise ValueError('policy catalog node count mismatch')
        for row in rows:
            yield from walk(directory, row, ancestors | {path}, paths)
    else:
        raise ValueError('policy catalog has an unknown node kind')


def read(directory, descriptor):
    return list(walk(directory, descriptor, set(), set()))


def files(directory, descriptor):
    paths = set()
    for _ in walk(directory, descriptor, set(), paths):
        pass
    return paths
