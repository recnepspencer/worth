"""Read Store's committed target roster, preserving its partition provenance."""
from pathlib import Path
import re


def scheduled_areas(repository):
    result = []
    for path in sorted((repository / '.github/workflows').glob('*.yml')):
        text = path.read_text(encoding='utf-8')
        if '--run-ignored only' not in text or 'workspaces/worth-store' not in text:
            continue
        package = re.search(r'-p\s+(worth-store[\w-]*)', text)
        target = re.search(r'--test\s+(\w+)', text)
        selected = re.search(r'test\(=([\w:]+)\)', text)
        if package and target and selected:
            result.append({'path': path, 'package': package[1], 'target': target[1],
                           'module': tuple(selected[1].split('::')[:-1]), 'selection': selected[0]})
    return result


def partitions(repository):
    source = repository / 'workspaces/worth-store/tools/store-test-runner/src/plan/integration_product.rs'
    if not source.exists():
        return {}, []
    text = source.read_text(encoding='utf-8')
    result = {}
    for label, body in re.findall(r'const (SCENARIO|UI|FORMAL)_TARGETS:.*?=\s*&\[(.*?)\];', text, re.S):
        for package, target in re.findall(r'target\(\s*"([^"]+)"\s*,\s*"([^"]+)"', body):
            result[(package, target)] = label.lower()
    base = 'workspaces/worth-store/tools/store-test-runner/src/plan/'
    table = [
        {'partition': 'owner-unit', 'category': 'unchanged', 'source': base + 'owner_product.rs',
         'reason': 'Selects --lib --bins --examples --benches and doctests; not integration targets.'},
        {'partition': 'scenario', 'category': 'ordinary; process override from source',
         'source': base + 'integration_product.rs::SCENARIO_TARGETS',
         'reason': 'Integration roster is behavioural, not an oracle claim; process/durability evidence takes precedence.'},
        {'partition': 'process-scenario', 'category': 'process', 'source': base + 'process_scenario_product.rs',
         'reason': 'Runs the separate store_process_scenario executable; leave the executable in place and report it.'},
        {'partition': 'ui', 'category': 'compile_fail', 'source': base + 'integration_product.rs::UI_TARGETS',
         'reason': 'Explicit compile-boundary roster; keep serial compile-boundary routing.'},
        {'partition': 'formal', 'category': 'oracle', 'source': base + 'integration_product.rs::FORMAL_TARGETS',
         'reason': 'Explicit formal model comparisons; process spawning takes the process override.'},
        {'partition': 'structural', 'category': 'unchanged', 'source': base + 'structural_product.rs',
         'reason': 'Runs repository tools, not integration test targets.'},
    ]
    for row in scheduled_areas(repository):
        table.append({'partition': 'scheduled ignored scale', 'category': 'cost',
                      'source': row['path'].relative_to(repository).as_posix(),
                      'reason': 'Scheduled workflow explicitly selects this ignored module: ' + '::'.join(row['module'])})
    return result, table
