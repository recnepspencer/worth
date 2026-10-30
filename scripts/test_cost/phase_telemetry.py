"""Aggregate optional process observations without changing any test outcome."""
import argparse
import json
import math
from pathlib import Path


FIELDS = {
    'test-cost-cpu': ('binary', 'test', 'cpu_seconds', 'wall_seconds', 'reserved_slots', 'accounting'),
    'test-cost-setup': ('binary', 'builder', 'fingerprint'),
}


def records(directory, pattern, kind):
    for path in sorted(Path(directory).glob(pattern)):
        try:
            lines = path.read_text(encoding='utf-8').splitlines()
        except OSError as error:
            yield {'type': 'test-cost-observation-error', 'path': str(path), 'reason': str(error)}
            continue
        for number, line in enumerate(lines, 1):
            try:
                row = json.loads(line)
                if not isinstance(row, dict) or row.get('type') != kind or any(
                        field not in row for field in FIELDS[kind]):
                    raise ValueError('incomplete observation schema')
                yield row
            except (ValueError, TypeError) as error:
                yield {'type': 'test-cost-observation-error', 'path': str(path),
                       'line': number, 'reason': str(error)}


def aggregate(cpu, setup, build_seconds, wall_seconds):
    yield from records(cpu, '*.json', 'test-cost-cpu')
    yield from records(setup, '*.jsonl', 'test-cost-setup')
    if all(math.isfinite(value) and value >= 0 for value in (build_seconds, wall_seconds)):
        yield {'type': 'test-cost-phases', 'build_seconds': build_seconds, 'wall_seconds': wall_seconds}
    else:
        yield {'type': 'test-cost-observation-error', 'reason': 'invalid phase clocks'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cpu', required=True, type=Path)
    parser.add_argument('--setup', required=True, type=Path)
    parser.add_argument('--build-seconds', required=True, type=float)
    parser.add_argument('--wall-seconds', required=True, type=float)
    args = parser.parse_args()
    for row in aggregate(args.cpu, args.setup, args.build_seconds, args.wall_seconds):
        print(json.dumps(row, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
