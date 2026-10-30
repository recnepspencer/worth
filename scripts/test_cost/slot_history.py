"""Verify portable CPU sample history against committed archive digests."""
import hashlib
import json
from pathlib import Path

from logs import read_lane


def archive_digest(raw):
    """Git may translate text line endings; all other archive bytes are exact."""
    return hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest()


def repository(directory):
    return next((parent for parent in (directory, *directory.parents) if (parent / '.git').exists()), None)


def resolve_source(name, directory):
    path = Path(name)
    root = repository(directory)
    return (root / path).resolve() if root is not None and not path.is_absolute() else path.resolve()


def read_source(path, directory):
    lane = read_lane(path)
    if any(lane['headers'][field] is None for field in ('Command', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code')):
        raise ValueError('telemetry source lacks complete retained command evidence: ' + str(path))
    if lane['headers']['Git status --short empty'] not in ('True', 'False'):
        raise ValueError('telemetry source has an invalid Git status observation')
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    root = repository(directory)
    name = str(path.relative_to(root)).replace('\\', '/') if root is not None and path.is_relative_to(root) else str(path)
    source = {'head': lane['headers']['HEAD'], 'head_scope': 'telemetry capture, not child execution revision',
              'telemetry_log': name, 'original_telemetry_log': str(path), 'sha256': digest,
              'command': json.loads(lane['headers']['Command']),
              'git_status_short_empty': lane['headers']['Git status --short empty'] == 'True',
              'start': lane['headers']['Start'], 'end': lane['headers']['End']}
    return source, lane['cpu']


def read_history(directory, manifest, explicit, execution_heads=None):
    sources = {}
    for source in manifest.get('observations', []):
        if type(source.get('git_status_short_empty')) is not bool:
            raise ValueError('committed telemetry provenance lacks its Git status observation')
        if source['sha256'] in sources:
            raise ValueError('sample source identity repeats in the committed index')
        sources[source['sha256']] = (dict(source), {})
    files, hashes = manifest.get('observation_files', []), manifest.get('observation_sha256', {})
    if set(files) != set(hashes):
        raise ValueError('sample archive digest inventory does not match its files')
    for name in files:
        path = (directory / name).resolve()
        if path.parent != directory:
            raise ValueError('sample archive escapes its owner')
        raw = path.read_bytes()
        if archive_digest(raw) != hashes[name]:
            raise ValueError('sample archive hash mismatch: ' + name)
        for line in raw.decode('utf-8').splitlines():
            sample = json.loads(line)
            identity, index = sample['capture_sha256'], sample['sample_index']
            if identity not in sources or type(index) is not int or index < 0:
                raise ValueError('invalid immutable CPU sample identity')
            events = sources[identity][1]
            if index in events:
                raise ValueError('immutable CPU sample identity repeats')
            events[index] = sample['observation']
    history = {}
    for identity, (source, indexed) in sources.items():
        if sorted(indexed) != list(range(len(indexed))):
            raise ValueError('CPU sample history has a missing index')
        events = [indexed[index] for index in sorted(indexed)]
        path = resolve_source(source['telemetry_log'], directory)
        if path.exists():
            actual, observed = read_source(path, directory)
            if actual['sha256'] != identity:
                raise ValueError('retained telemetry source changed: ' + str(path))
            if observed != events:
                raise ValueError('sample archive disagrees with retained telemetry')
        history[identity] = (source, events)
    for path in explicit:
        source, events = read_source(path, directory)
        identity = source['sha256']
        if identity in history:
            if history[identity][1] != events:
                raise ValueError('retained CPU sample identity contradicts its archive')
            source = history[identity][0]
        head = (execution_heads or {}).get(str(path))
        if head is not None:
            source.setdefault('execution_start_head', head)
        history[identity] = (source, events)
    return [history[identity] for identity in sorted(history)]
