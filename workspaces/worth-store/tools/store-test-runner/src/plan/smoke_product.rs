use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use crate::product::{smoke_cases, SmokeCase};

use super::TestExecutionUnit;

pub(super) fn smoke(workspace_root: &Path) -> Vec<TestExecutionUnit> {
    let mut packages = BTreeMap::<&str, Vec<_>>::new();
    for case in smoke_cases() {
        packages.entry(case.package).or_default().push(case);
    }
    packages
        .into_iter()
        .map(|(package, cases)| smoke_package(package, &cases, workspace_root))
        .collect()
}

fn smoke_package(package: &str, cases: &[&SmokeCase], workspace_root: &Path) -> TestExecutionUnit {
    let targets = cases
        .iter()
        .map(|case| case.target)
        .collect::<BTreeSet<_>>();
    let features = cases
        .iter()
        .filter_map(|case| case.feature)
        .collect::<BTreeSet<_>>();
    let selectors = cases
        .iter()
        .map(|case| {
            format!(
                "(binary_id(={}::{}) & test(={}))",
                case.package, case.target, case.filter
            )
        })
        .collect::<Vec<_>>();

    TestExecutionUnit::cargo(
        format!("smoke::{package}"),
        workspace_root,
        smoke_arguments(package, targets, features, selectors),
    )
}

fn smoke_arguments(
    package: &str,
    targets: BTreeSet<&str>,
    features: BTreeSet<&str>,
    selectors: Vec<String>,
) -> Vec<String> {
    let mut arguments = vec![
        "nextest".into(),
        "run".into(),
        "-p".into(),
        package.into(),
        "--no-fail-fast".into(),
        "--no-tests=fail".into(),
    ];
    for target in targets {
        arguments.extend(["--test".into(), target.into()]);
    }
    for feature in features {
        arguments.extend(["--features".into(), feature.into()]);
    }
    arguments.extend(["--filterset".into(), selectors.join(" + ")]);
    arguments
}
