"""Generate category-prefix routing for the existing Query gate and Store runner."""
import json
import re

from court_roots import module_path
from store_partitions import scheduled_areas


def selectors(effects, root, workspace, courts, roster, executables=()):
    result = []
    if workspace == root:
        path = root / 'scripts/ci/query_gate_courts.py'
        if path.exists():
            text = path.read_text(encoding='utf-8')
            start, end = text.index('ROOT_TESTS = ['), text.index('\nGROUPS =', text.index('ROOT_TESTS = ['))
            owners = {c['name']: sorted({u['owner']['cargo']['name'] for u in c['units']}) for c in courts}
            replacement = 'ROOT_TESTS = ["--workspace", "--lib", "--tests"]\nCOURT_OWNERS = ' + repr(owners) + '\n'
            text = text[:start] + replacement + text[end:]
            text = text.replace('return lambda binary: binary["package"] in names\n',
                                'return lambda binary: bool(set(names) & set(COURT_OWNERS.get(binary["package"], [binary["package"]])))\n')
            text = text.replace('for prefix in ("query", "bank"):', 'for prefix in ("query", "bank", "root"):')
            effects.write(path, text)
            result.append({'path': str(path.relative_to(root)).replace('\\', '/'),
                           'reason': 'Select root courts through their original owning packages and top-level category prefixes.'})
    else:
        path = workspace / 'tools/store-test-runner/src/plan/integration_product.rs'
        if path.exists():
            text = store_runner(courts, executables)
            effects.write(path, text)
            result.append({'path': str(path.relative_to(root)).replace('\\', '/'),
                           'reason': 'Replace binary-per-file rosters with court binaries and category-prefix partitions; retain sharding and certification dependency features.'})
        for row in scheduled_areas(root):
            unit = next(u for c in courts for u in c['units'] if u['owner']['cargo']['name'] == row['package']
                        and u['owner']['targets'][u['target_index']]['cargo']['name'] == row['target']
                        and u['node'].names == row['module'])
            text = row['path'].read_text(encoding='utf-8')
            text = text.replace('-p ' + row['package'], '-p ' + unit['court'])
            text = text.replace('--test ' + row['target'], '--test ' + unit['court'].replace('-', '_'))
            text = text.replace(row['selection'], 'test(/^' + '::'.join(module_path(unit)) + '::/)')
            text = re.sub(r'--features\s+([\w-]+)', lambda m: '--features ' + row['package'] + '/' + m[1], text)
            effects.write(row['path'], text)
            result.append({'path': row['path'].relative_to(root).as_posix(), 'reason': 'Select the existing ignored scale module through its new cost/category prefix.'})
    # Workflows invoking these entry points retain their lane names and commands.
    # The external orchestration/affected_lanes.py is not versioned in this tree.
    return result


def store_runner(courts, executables=()):
    rows = []
    for court in courts:
        categories = {u['category'].split('::')[0] if u['category'] else 'ordinary' for u in court['units']}
        rows.append('    (' + json.dumps(court['name']) + ', ' + json.dumps(court['name'].replace('-', '_'))
                    + ', &[' + ', '.join(json.dumps(c) for c in sorted(categories)) + ']),')
    builds = ['build', '--bins'] + [arg for package in sorted({r['package'] for r in executables}) for arg in ('-p', package)]
    build_args = ', '.join(json.dumps(arg) + '.into()' for arg in builds)
    prerequisite = 'TestExecutionUnit::cargo(format!("{lane}::executables"), root, vec![' + build_args + ']), ' if executables else ''
    return '''//! Court selections generated from the applied court plan. Categories are prefixes.
use std::path::Path;
use super::TestExecutionUnit;

pub(super) fn scenario(shard: Option<(usize, usize)>, root: &Path) -> Vec<TestExecutionUnit> {
    units("scenario", &["ordinary", "process", "determinism"], shard, root)
}
pub(super) fn ui(shard: Option<(usize, usize)>, root: &Path) -> Vec<TestExecutionUnit> {
    units("ui", &["compile_fail"], shard, root)
}
pub(super) fn formal(shard: Option<(usize, usize)>, root: &Path) -> Vec<TestExecutionUnit> {
    units("formal", &["oracle"], shard, root)
}
fn units(lane: &str, categories: &[&str], shard: Option<(usize, usize)>, root: &Path) -> Vec<TestExecutionUnit> {
    let mut args: Vec<String> = ["nextest", "run", "--workspace", "--tests", "--no-fail-fast", "--no-tests=fail", "--filterset"]
        .into_iter().map(str::to_owned).collect();
    let mut filters = Vec::new();
    for &(package, binary, present) in COURTS {
        for &category in categories {
            if !present.contains(&category) { continue; }
            let prefix = match category {
                "ordinary" => "not (test(/^process::/) + test(/^oracle::/) + test(/^cost::/) + test(/^determinism::/) + test(/^compile_fail::/))".to_owned(),
                "process" | "oracle" | "cost" | "determinism" | "compile_fail" => format!("test(/^{category}::/)"),
                other => panic!("unregistered category {other}"),
            };
            filters.push(format!("(binary_id(={package}::{binary}) & ({prefix}))"));
        }
    }
    args.push(filters.join(" + "));
    if let Some((index, count)) = shard {
        args.extend(["--partition".into(), format!("hash:{}/{}", index + 1, count)]);
    }
    vec![EXECUTABLE_PREREQUISITETestExecutionUnit::cargo(format!("{lane}::nextest"), root, args)]
}
const COURTS: &[(&str, &str, &[&str])] = &[
''' .replace('EXECUTABLE_PREREQUISITE', prerequisite) + '\n'.join(rows) + '\n];\n'
