"""Index retained lane evidence without inventing test counts or combining runs."""
import argparse
import hashlib
import json
from pathlib import Path
import re

from logs import read_lane
from telemetry import timing_summary


def entry(path):
    path = Path(path)
    raw = path.read_bytes()
    text = re.sub(r'\r+\n', '\n', raw.decode('utf-8-sig', errors='replace'))
    lane = read_lane(path)
    # Parent commands can print child headers. The enclosing command owns only
    # the leading header block and trailing footer, never those nested lines.
    header = text.split('\n\n', 1)[0]
    fields = {}
    for name in ('Command', 'HEAD', 'Git status --short empty', 'Start'):
        match = re.search(rf'^{re.escape(name)}: (.+)$', header, re.M)
        fields[name] = match[1].strip() if match else None
    fields.update({name: lane['headers'][name] for name in ('End', 'Exit code')})
    counts, basis = None, 'no test result observation'
    if lane['tests'] or lane['suites'] or lane['plain_test_counts'] is not None:
        summary = timing_summary(lane)
        counts = {name: summary[name] for name in ('pass', 'fail', 'ignored')}
        basis = 'terminal test events' if lane['tests'] else 'aggregate test summaries'
    else:
        # unittest's runner writes a different, stable summary. Never infer
        # success from its exit alone, especially for non-test commands.
        result = re.findall(r'^Ran (\d+) tests? in .+$', text, re.M)
        verdicts = re.findall(r'^(OK(?: \([^\n]+\))?|FAILED \([^\n]+\))\r?$', text, re.M)
        if result and verdicts:
            verdict = verdicts[-1]
            failures = sum(int(number) for number in re.findall(r'(?:failures|errors)=(\d+)', verdict))
            skipped = sum(int(number) for number in re.findall(r'skipped=(\d+)', verdict))
            counts = {'pass': int(result[-1]) - failures - skipped,
                      'fail': failures, 'ignored': skipped}
            basis = 'unittest aggregate summary; subtest failures may exceed test methods'
    return {'log': str(path), 'sha256': hashlib.sha256(raw).hexdigest(),
            'headers': fields, 'complete_header': all(value is not None for value in fields.values()),
            'counts': counts, 'count_basis': basis,
            'timing_telemetry': lane['telemetry']}


def markdown(rows):
    lines = ['# Lane evidence index', '',
             'Each row is a separate capture. Parent wrappers and child captures overlap; '
             'do not add their counts. Unknown counts are not zero. Failed attempts remain visible.', '',
             '| Log | HEAD | Clean | Exit | Pass / fail / ignored |',
             '|---|---|---|---:|---|']
    for row in rows:
        headers, counts = row['headers'], row['counts']
        rendered = 'unknown' if counts is None else ' / '.join(str(counts[key]) for key in ('pass', 'fail', 'ignored'))
        lines.append(f'| [{Path(row["log"]).name}]({Path(row["log"]).as_posix()}) | '
                     f'{(headers["HEAD"] or "unknown")[:12]} | {headers["Git status --short empty"]} | '
                     f'{headers["Exit code"]} | {rendered} |')
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, default=Path('target/lane-evidence'))
    parser.add_argument('--output', type=Path, default=Path('target/test-cost/evidence-index'))
    args = parser.parse_args()
    rows = [entry(path) for path in sorted(args.directory.glob('*.log'))]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.with_suffix('.json').write_text(json.dumps(rows, indent=2) + '\n', encoding='utf-8')
    args.output.with_suffix('.md').write_text(markdown(rows), encoding='utf-8')
    print(json.dumps({'captures': len(rows), 'incomplete_headers': sum(not row['complete_header'] for row in rows),
                      'nonzero_exits': sum(row['headers']['Exit code'] not in ('0', None) for row in rows)}))


if __name__ == '__main__':
    main()
