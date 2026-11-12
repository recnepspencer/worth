"""CLI routing for legacy suites and the source-bound affected Root path."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from query_gate_cost import DOC_SUITES, SUITE_ALIASES
from query_gate_selection import select_suites, workspace_coverage
from query_gate_prebuild import refused_scale_suites
from query_gate_normal import compile_plan, assert_unchanged, command_plan


def main(driver, argv=None, default_suites=None, default_out=None, allowed_suites=None):
    ROOT, ORDER = driver.ROOT, driver.ORDER
    TEST_SUITES, COMMAND_SUITES = driver.TEST_SUITES, driver.COMMAND_SUITES
    parser = argparse.ArgumentParser(description=driver.__doc__.split("\n\n")[0])
    parser.add_argument("suites", nargs="?",
                        help="suite names or root-landing, query-landing, bank-landing, doc")
    parser.add_argument("--full", action="store_true", help="the complete existing gate (Root wrapper stays Root-only)")
    parser.add_argument("--out", type=Path, default=default_out or ROOT / "target" / "query-gate")
    parser.add_argument("--list", action="store_true", help="print suite names")
    parser.add_argument("--census", action="store_true", help="refuse scheduled scale suites")
    parser.add_argument("--changed-since", help="affected Root inputs since full fetched master identity")
    args = parser.parse_args(argv)
    if args.changed_since is not None:
        if args.suites is not None or args.full or args.list or args.census or allowed_suites is not None:
            parser.error("--changed-since requires its own unmodified normal request")
        try:
            plan = compile_plan(ROOT, args.changed_since)
            return run_normal(driver, args.out, plan)
        except (ValueError, OSError, subprocess.CalledProcessError) as error:
            parser.error(str(error))
    if args.list:
        print("\n".join(allowed_suites or dict.fromkeys([*ORDER, *TEST_SUITES, *SUITE_ALIASES, *COMMAND_SUITES])))
        return 0
    try:
        suites = select_suites(args.suites, args.full, default_suites, ORDER, ROOT, Path.cwd())
    except ValueError as error:
        parser.error(str(error))
    unknown = [name for name in suites if name not in TEST_SUITES and name not in COMMAND_SUITES and name not in DOC_SUITES]
    if unknown:
        parser.error("unknown suite: " + ", ".join(unknown))
    if len(set(suites)) != len(suites):
        parser.error("suite repeats in the execution plan")
    if allowed_suites is not None and any(name not in allowed_suites for name in suites):
        parser.error("suite is outside this workspace gate")
    if refused_scale_suites(suites, args.census):
        parser.error("ignored scale tests are scheduled-only, outside census execution")
    if args.census and "ui" in suites:
        parser.error("worth-ui is excluded from census execution")
    args.out.mkdir(parents=True, exist_ok=True)
    gate = driver.Gate(args.out, census=args.census)
    gate.plan(suites)
    red, results = [], []
    for suite in suites:
        result = gate.run(suite)
        results.append(result)
        brief = {key: value for key, value in result.items()
                 if key not in ("chunks", "commands", "binaries")}
        print("GATE", json.dumps(brief), flush=True)
        if not result["ok"]:
            red.append(suite)
            if suite == "fmt":
                break
    summary = {"status": "red" if red else "green", "requested_suites": suites,
               "workspace_coverage": workspace_coverage(suites, results)}
    (args.out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
    print("GATE", json.dumps(summary), flush=True)
    return 1 if red else 0



def run_normal(driver, output, plan):
    output = output.resolve()
    if output.exists():
        raise ValueError('affected normal requires a fresh output directory')
    if output.is_relative_to(driver.ROOT.resolve()) and not output.is_relative_to((driver.ROOT / 'target').resolve()):
        raise ValueError('normal output inside the checkout must be under target')
    output.mkdir(parents=True)
    gate = driver.Gate(output, affected=plan)
    gate.plan(plan['suites'])
    commands = command_plan(plan, driver)
    results = []
    for name in [*commands, *plan['suites']]:
        assert_unchanged(driver.ROOT, plan)
        if name in commands:
            rows = [gate.command(label, argv) for label, argv in commands[name]]
            result = dict(suite=name, normal_plan=plan, commands=rows,
                          ok=all(row['exit'] == 0 for row in rows))
            (output / (name + '.json')).write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
        else:
            result = gate.run(name)
        result['application_build'] = gate.env['WORTH_IMPLEMENTATION_BUILD_ID']
        (output / (name + '.json')).write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
        results.append(result)
        print('GATE', json.dumps({key: value for key, value in result.items()
                                 if key not in ('chunks', 'commands', 'binaries', 'normal_plan')}), flush=True)
        if not result['ok']:
            break
    assert_unchanged(driver.ROOT, plan)
    # Bind the existing run outputs, not historical executions or push permission.
    artifacts = {path.relative_to(output).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                 for path in sorted(output.rglob('*')) if path.is_file()}
    required = [*commands, *plan['suites']]
    summary = dict(schema=plan['schema'], normal_plan=plan, status=(
        'green' if len(results) == len(required) and all(row['ok'] for row in results) else 'red'),
        label='SOURCE-ONLY / NO RUST COURT COVERAGE' if plan['mode'] == 'source-only' else 'AFFECTED NORMAL / PARTIAL ROOT',
        application_build=gate.env['WORTH_IMPLEMENTATION_BUILD_ID'], output=str(output), requested_suites=required,
        command_plan=commands, workspace_coverage={'root': 'partial'} if plan['packages'] else {},
        untested='All unselected Root owners and all other workspaces; no full-workspace qualification.',
        artifacts=artifacts)
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
    print('GATE', json.dumps(summary), flush=True)
    return 0 if summary['status'] == 'green' else 1
