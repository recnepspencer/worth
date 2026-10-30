"""Derive prerequisite graphs from the tests actually selected for execution."""
from compiler_prebuild import CompilerScope


def required(workspace, rows):
    rows = [row for row in rows if row.get('selected', True) and not row['ignored']]
    fixtures = any(row['test'].startswith('compile_fail::') for row in rows)
    units = workspace == 'store' and any(row['binary'] == 'worth-store-test-support'
                                         and row['test'].startswith('compiler_boundary::') for row in rows)
    if fixtures and units:
        scope = CompilerScope.ALL
    elif fixtures:
        scope = CompilerScope.FIXTURES
    elif units:
        scope = CompilerScope.UNITS
    else:
        scope = CompilerScope.NONE
    proof = workspace == 'root' and any(row['binary'].startswith('worth-court-proof')
                                        and 'band_guard_cargo_contract::' in row['test'] for row in rows)
    return scope, proof
