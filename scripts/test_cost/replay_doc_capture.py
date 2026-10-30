"""Derive doctest observations from a completed ordered Cargo capture, offline."""
import argparse
import hashlib
import json
from pathlib import Path
import re

from doc_contracts import (DocContractFailure, DocFailureKind, owners_from_metadata,
                           parse_output, retain_observation)


def frame(path):
    path = Path(path).resolve()
    text = path.read_text(encoding='utf-8')
    headers = {}
    for name in ('Command', 'Working directory', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code'):
        values = re.findall(r'^' + re.escape(name) + r': (.+)$', text, re.M)
        if len(values) != 1:
            raise DocContractFailure(DocFailureKind.FRAMING, f'{path}: expected exactly one {name} header')
        headers[name] = values[0]
    command = json.loads(headers['Command'])
    if (command[:2] != ['cargo', 'test'] or any(flag not in command for flag in
            ('--workspace', '--doc', '--no-fail-fast', '--locked', '--manifest-path', '--message-format=json')) or
            command[-3:] != ['--', '--format', 'pretty'] or any(flag in command for flag in
            ('--ignored', '--include-ignored', '--exclude', '-p', '--package'))):
        raise DocContractFailure(DocFailureKind.UNSAFE_SELECTION, 'not the canonical full-workspace doc command')
    markers = []
    offset = 0
    for line in text.splitlines(keepends=True):
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            value = None
        if isinstance(value, dict) and value.get('type') == 'cargo-invocation':
            markers.append((value, offset + len(line)))
        offset += len(line)
    if len(markers) != 1 or markers[0][0].get('capture') != 'ordered' or markers[0][0].get('command') != command:
        raise DocContractFailure(DocFailureKind.FRAMING, f'{path}: no unique ordered command capture')
    footer = re.search(r'^End: .+\nExit code: -?\d+\s*(?:\n|$)', text, re.M)
    if footer is None or footer.start() < markers[0][1]:
        raise DocContractFailure(DocFailureKind.FRAMING, f'{path}: command has no complete footer')
    base = text[:footer.end()].rstrip() + '\n'
    return {'path': str(path), 'headers': headers, 'command': command,
            'body': text[markers[0][1]:footer.start()], 'exit_code': int(headers['Exit code']),
            'capture_sha256': hashlib.sha256(base.encode('utf-8')).hexdigest()}


def replay(capture, output_directory, metadata_path, canonical_capture=None):
    observed = frame(capture)
    metadata_path = Path(metadata_path).resolve()
    metadata_text = metadata_path.read_text(encoding='utf-8')
    metadata = json.loads(metadata_text)
    command = observed['command']
    cwd = Path(observed['headers']['Working directory']).resolve()
    manifest = Path(command[command.index('--manifest-path') + 1])
    manifest = (cwd / manifest).resolve() if not manifest.is_absolute() else manifest.resolve()
    if manifest.parent != Path(metadata['workspace_root']).resolve():
        raise DocContractFailure(DocFailureKind.MISSING_OWNER, 'captured metadata belongs to another workspace')
    destinations = [Path(capture)]
    if canonical_capture is not None:
        canonical = frame(canonical_capture)
        if canonical['capture_sha256'] != observed['capture_sha256']:
            raise DocContractFailure(DocFailureKind.FRAMING, 'canonical log differs from the original command capture')
        destinations.append(Path(canonical_capture))
    parsed = parse_output(observed['body'], '', owners_from_metadata(metadata))
    parsed['provenance'] = {key: observed[key] for key in ('path', 'headers', 'command', 'capture_sha256')}
    parsed['provenance'].update(metadata=str(metadata_path),
                                metadata_sha256=hashlib.sha256(metadata_text.encode('utf-8')).hexdigest(),
                                canonical_capture=str(Path(canonical_capture).resolve()) if canonical_capture else None,
                                execution='retained-command-replay; no Cargo execution')
    result = retain_observation(parsed, observed['exit_code'], output_directory, destinations)
    (Path(output_directory) / 'replay.json').write_text(json.dumps(result['provenance'], indent=2) + '\n', encoding='utf-8')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--canonical-capture', type=Path)
    parser.add_argument('--metadata', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = replay(args.capture, args.output, args.metadata, args.canonical_capture)
    print(json.dumps({'status': result['status'], 'counts': result['counts'], 'owners': len(result['owners']),
                      'blocks': len(result['blocks']), 'provenance': result['provenance']}, indent=2))
    return 0 if result['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
