"""Observe rustdoc contracts omitted by nextest, without running ignored snippets."""
from collections import Counter
from enum import Enum
from functools import lru_cache
import json
import os
from pathlib import Path
import re
import tomllib

from workspace import members
from filesystem_path import filesystem_path


class DocFailureKind(Enum):
    UNKNOWN_OWNER = 'unknown-owner'
    AMBIGUOUS_OWNER = 'ambiguous-owner'
    MISSING_OWNER = 'missing-owner'
    FRAMING = 'invalid-framing'
    COUNTS = 'inconsistent-counts'
    DUPLICATE = 'duplicate-test'
    UNSAFE_SELECTION = 'ignored-selection'
    MISSING_LOG = 'missing-command-log'


class DocContractFailure(ValueError):
    def __init__(self, kind, context):
        self.kind = kind
        super().__init__(f'{kind.value}: {context}')


HEADING = re.compile(r'^\s*Doc-tests\s+(\S+)\s*$')
START = re.compile(r'^running (\d+) tests?(?: \(shuffle seed: \d+\))?$')
ROW = re.compile(r'^test (.+?)\s+\.\.\. (ok|FAILED(?: \(time limit exceeded\))?|ignored(?:,.*)?)$')
SUMMARY = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; '
                     r'(\d+) measured; (\d+) filtered out(?:; finished in ([\d.]+)s)?$')
MERGED = re.compile(r'^all doctests ran in [\d.]+s; merged doctests compilation took [\d.]+s$')
ANSI = re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]')
MODES = re.compile(r' - (compile fail|compile|should panic)$')


def owners_for(manifest):
    """Cargo headings identify library targets; package names remain the identity."""
    owners = {}
    for package in members(manifest):
        path = package['manifest']
        configuration = tomllib.loads(filesystem_path(path).read_text(encoding='utf-8-sig'))
        library = configuration.get('lib', {})
        if not library.get('doctest', True):
            continue
        source = path.parent / library.get('path', 'src/lib.rs')
        explicit = 'lib' in configuration
        if not explicit and not configuration['package'].get('autolib', True):
            continue
        if not filesystem_path(source).is_file():
            if explicit:
                raise DocContractFailure(DocFailureKind.MISSING_OWNER, f'library source missing: {source}')
            continue
        crate = library.get('name', package['name'].replace('-', '_'))
        if crate in owners:
            raise DocContractFailure(DocFailureKind.AMBIGUOUS_OWNER, f'Doc-tests {crate}')
        owners[crate] = package['name']
    return owners


def clean(text):
    if isinstance(text, bytes):
        text = text.decode('utf-8', errors='replace')
    return ANSI.sub('', text or '').splitlines()


def owner_name(crate, owners):
    if crate not in owners:
        raise DocContractFailure(DocFailureKind.UNKNOWN_OWNER, f'Doc-tests {crate}')
    return owners[crate]


def finish_block(block, summary):
    counts = Counter(row['event'] for row in block['rows'])
    passed, failed, ignored, measured, filtered = map(int, summary.group(2, 3, 4, 5, 6))
    observed = (counts['ok'], counts['failed'], counts['ignored'])
    if (observed != (passed, failed, ignored) or measured or filtered or
            block['declared'] != passed + failed + ignored):
        raise DocContractFailure(DocFailureKind.COUNTS,
                                 f'block {block["index"]}: declared={block["declared"]}, '
                                 f'observed={observed}, summary={summary.group(0)}')
    if (summary[1] == 'ok') != (failed == 0):
        raise DocContractFailure(DocFailureKind.COUNTS, 'summary outcome contradicts failures')
    block['summary'] = {'passed': passed, 'failed': failed, 'ignored': ignored}


def framed_blocks(stdout, owners):
    """Keep real in-stream heading boundaries when CargoRunner captures ordered output.

    Rust 1.94 can emit several merged edition/attribute groups and standalone
    cases for one heading. The final merged timing line closes that owner.
    """
    blocks, headings, active, current = [], [], None, None
    for line in clean(stdout):
        if heading := HEADING.fullmatch(line):
            if active is not None:
                raise DocContractFailure(DocFailureKind.FRAMING, 'heading before block completion')
            current = owner_name(heading[1], owners)
            headings.append(current)
            continue
        if start := START.fullmatch(line):
            if active is not None:
                raise DocContractFailure(DocFailureKind.FRAMING, 'nested or truncated libtest block')
            active = {'index': len(blocks), 'declared': int(start[1]), 'owner': current,
                      'rows': [], 'merged_end': False, 'diagnostics': False}
            continue
        if summary := SUMMARY.fullmatch(line):
            if active is None:
                raise DocContractFailure(DocFailureKind.FRAMING, 'summary without running block')
            finish_block(active, summary)
            blocks.append(active)
            active = None
            continue
        if line.startswith('test result:'):
            raise DocContractFailure(DocFailureKind.FRAMING, f'incomplete summary: {line}')
        if MERGED.fullmatch(line):
            if active is not None or not blocks or blocks[-1]['merged_end']:
                raise DocContractFailure(DocFailureKind.FRAMING, 'unpaired merged-doctest marker')
            blocks[-1]['merged_end'] = True
            continue
        if active is not None and (line == 'failures:' or line.startswith('---- ')):
            active['diagnostics'] = True
        if active is not None and active['diagnostics']:
            continue
        if row := ROW.fullmatch(line):
            if active is None:
                raise DocContractFailure(DocFailureKind.FRAMING, 'test result outside running block')
            name = MODES.sub('', row[1]).rstrip()
            if not re.fullmatch(r'.+ \(line \d+\)', name):
                raise DocContractFailure(DocFailureKind.FRAMING, f'not an exact rustdoc test name: {name}')
            event = 'ignored' if row[2].startswith('ignored') else 'failed' if row[2].startswith('FAILED') else 'ok'
            active['rows'].append({'test': name, 'event': event})
    if active is not None:
        raise DocContractFailure(DocFailureKind.FRAMING, 'running block has no complete summary')
    return blocks, headings


def separated_assignment(blocks, headings):
    """Use separate streams only if contiguous block grouping is unambiguous."""
    @lru_cache(maxsize=None)
    def partitions(owner, start):
        if owner == len(headings):
            return ((),) if start == len(blocks) else ()
        choices = []
        for end in range(start, len(blocks)):
            if end > start and blocks[end - 1]['merged_end']:
                break
            if end != start and not blocks[end]['merged_end']:
                continue
            for tail in partitions(owner + 1, end + 1):
                choices.append(((start, end + 1), *tail))
                if len(choices) == 2:
                    return tuple(choices)
        return tuple(choices)

    choices = partitions(0, 0)
    if len(choices) != 1:
        kind = DocFailureKind.AMBIGUOUS_OWNER if choices else DocFailureKind.FRAMING
        raise DocContractFailure(kind, 'separate rustdoc streams do not prove unique owner grouping')
    for owner, (start, end) in zip(headings, choices[0]):
        for block in blocks[start:end]:
            block['owner'] = owner


def parse_output(stdout, stderr, owners):
    blocks, headings = framed_blocks(stdout, owners)
    separate = [owner_name(m[1], owners) for line in clean(stderr) if (m := HEADING.fullmatch(line))]
    if headings and separate:
        raise DocContractFailure(DocFailureKind.AMBIGUOUS_OWNER, 'owner headings split across streams')
    ordered = bool(headings)
    headings = headings or separate
    if len(headings) != len(set(headings)):
        raise DocContractFailure(DocFailureKind.AMBIGUOUS_OWNER, 'a library heading occurred more than once')
    if set(headings) != set(owners.values()):
        raise DocContractFailure(DocFailureKind.MISSING_OWNER, 'not every workspace library has a Doc-tests heading')
    if not ordered:
        separated_assignment(blocks, headings)
    if {b['owner'] for b in blocks} != set(headings):
        raise DocContractFailure(DocFailureKind.FRAMING, 'a library heading has no complete libtest block')
    inventory, events, seen = [], [], set()
    for block in blocks:
        for row in block['rows']:
            binary = block['owner'] + '::doc'
            identity = (binary, row['test'])
            if identity in seen:
                raise DocContractFailure(DocFailureKind.DUPLICATE, repr(identity))
            seen.add(identity)
            inventory.append({'binary': binary, 'test': row['test'], 'ignored': row['event'] == 'ignored'})
            events.append({'type': 'test', 'event': row['event'], 'name': binary + '$' + row['test'],
                           'test-cost': {'execution': 'rustdoc', 'block': block['index']}})
    return {'inventory': sorted(inventory, key=lambda r: (r['binary'], r['test'])),
            'events': events, 'blocks': blocks, 'owners': headings}


def execute(manifest, output_directory, runner, profile='dev'):
    manifest, output = Path(manifest).resolve(), Path(output_directory).resolve()
    owners = owners_for(manifest)
    environment = getattr(runner, 'environment', None)
    if environment is None:
        environment = os.environ
    for name in ('RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS'):
        if any(flag in environment.get(name, '') for flag in ('--ignored', '--include-ignored')):
            raise DocContractFailure(DocFailureKind.UNSAFE_SELECTION, name)
    output.mkdir(parents=True, exist_ok=True)
    log = output / 'doctests.log'
    command = ['cargo', 'test', '--workspace', '--doc', '--profile', profile, '--no-fail-fast',
               '--locked', '--manifest-path', str(manifest), '--message-format=json',
               '--', '--format', 'pretty']
    result = runner(command, manifest.parent, log)
    parsed = parse_output(result.stdout, result.stderr, owners)
    parsed['selections'] = [row for row in parsed['inventory'] if not row['ignored']]
    parsed['exit_code'] = result.returncode
    parsed['counts'] = {event: sum(row['event'] == event for row in parsed['events'])
                        for event in ('ok', 'failed', 'ignored')}
    parsed['status'] = 'pass' if result.returncode == 0 and not any(
        event['event'] == 'failed' for event in parsed['events']) else 'fail'
    destinations = {log.resolve()}
    if getattr(runner, 'logs', []):
        destinations.add(Path(runner.logs[-1]).resolve())
    for destination in destinations:
        if not destination.is_file():
            raise DocContractFailure(DocFailureKind.MISSING_LOG, str(destination))
        with destination.open('a', encoding='utf-8') as stream:
            stream.write('\n' + ''.join(json.dumps(event) + '\n' for event in parsed['events']))
    for name in ('inventory', 'selections'):
        (output / f'{name}.json').write_text(json.dumps(parsed[name], indent=2) + '\n', encoding='utf-8')
    (output / 'result.json').write_text(json.dumps(parsed, indent=2) + '\n', encoding='utf-8')
    return parsed
