"""Render the deterministic plan and disjoint fixer handoff for human review."""
import json


def markdown(plan):
    rows = ['# Court migration plan', '', '`' + plan['workspace_manifest'] + '`', '',
            'Planning only. Source names and test bodies are preserved; path/module edits are listed in JSON.', '',
            '## Totals', '', '```json', json.dumps(plan['totals'], indent=2, sort_keys=True), '```', '',
            '## Courts', '', '| Court | Product frontier | Rust lines | Categories |',
            '|---|---|---:|---|']
    for court in plan['courts']:
        rows.append('| ' + court['name'] + ' | ' + ', '.join(court['frontier']) + ' | ' + str(court['rust_lines'])
                    + ' | ' + ', '.join(court['categories']) + ' |')
    rows += ['', '## Executable destinations and separation', '', '```json', json.dumps(plan['binary_constraints'], indent=2, sort_keys=True), '```']
    rows += ['', '## Testkits', '']
    for kit in plan['testkits']:
        rows.append('- ' + kit['name'] + ': ' + str(kit['rust_lines']) + ' Rust lines; readers: ' + ', '.join(kit['readers']))
    rows += ['', '## Category mapping', '', '```json', json.dumps(plan['category_mapping'], indent=2, sort_keys=True), '```', '',
             '## Splits', '', '```json', json.dumps(plan['splits'], indent=2, sort_keys=True), '```', '',
             '## Dev-dependencies to drop', '']
    for drop in plan['dev_dependencies_to_drop']:
        rows.append('- ' + drop['package'] + ': ' + (', '.join(drop['dependencies']) or '(none)'))
    rows += ['', '## Path and module rewrites', '', str(len(plan['path_rewrites'])) + ' recorded rewrites in court-plan.json.', '',
             '## Fixer work by court', '']
    for name, work in plan['fixer_work_by_court'].items():
        rows += ['### ' + name, '']
        for item in work:
            rows.append('- **' + item['kind'] + '** `' + item['source'] + '`: ' + item['reason']
                        + (' Symbols: ' + ', '.join('`' + s + '`' for s in item['symbols']) if item['symbols'] else ''))
        if not work:
            rows.append('No predicted semantic repairs.')
        rows.append('')
    if plan['blockers']:
        rows += ['## Apply blockers', '', '```json', json.dumps(plan['blockers'], indent=2, sort_keys=True), '```', '']
    return '\n'.join(rows)
