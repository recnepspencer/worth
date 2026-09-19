use super::*;

#[test]
fn effect_closeout_without_dependency_retirement_settles_into_the_effect_parent() {
    let mut fixture = effect_closeout_fixture();
    // The dependency branch stays alive, so it is the nearest surviving
    // ancestor: the root behind it is not where this effect's work belongs.
    let mut request = closeout_request(
        &fixture,
        fixture
            .shell
            .worker_branch_basis(fixture.main.id.0)
            .unwrap(),
    );
    request.dependency_basis_retirement = None;
    let denial = fixture
        .shell
        .closeout_worker_effect_branch(request)
        .unwrap_err();
    assert!(
        denial.message.contains(&format!(
            "nearest surviving ancestor `{}`",
            fixture.dependency.branch.id.0
        )),
        "{}",
        denial.message
    );

    // The denial changed nothing: both snapshots are still restorable.
    fixture
        .shell
        .restore_branch_snapshot(fixture.effect.branch.id.0, fixture.effect_snapshot.clone())
        .expect("target denial must preserve effect snapshot authority");
    fixture
        .shell
        .restore_branch_snapshot(
            fixture.dependency.branch.id.0,
            fixture.dependency_snapshot.clone(),
        )
        .expect("target denial must preserve dependency snapshot authority");

    let dependency_id = fixture.dependency.branch.id.0;
    let mut request = closeout_request(
        &fixture,
        fixture.shell.worker_branch_basis(dependency_id).unwrap(),
    );
    request.canonical_transaction.branch_id = dependency_id;
    request.dependency_basis_retirement = None;
    fixture
        .shell
        .closeout_worker_effect_branch(request)
        .expect("closeout settles into the live dependency branch");
    fixture.shell.switch_branch(dependency_id).unwrap();
    assert_eq!(
        fixture.shell.read_value("counter").unwrap(),
        SignalValue::Number(23.0)
    );
    // The effect branch is retired by the closeout; only the dependency
    // (the canonical target) survives with its snapshot authority.
    assert!(fixture
        .shell
        .restore_branch_snapshot(fixture.effect.branch.id.0, fixture.effect_snapshot)
        .is_err());
    fixture
        .shell
        .restore_branch_snapshot(fixture.dependency.branch.id.0, fixture.dependency_snapshot)
        .expect("the surviving dependency branch keeps its snapshot authority");
}
