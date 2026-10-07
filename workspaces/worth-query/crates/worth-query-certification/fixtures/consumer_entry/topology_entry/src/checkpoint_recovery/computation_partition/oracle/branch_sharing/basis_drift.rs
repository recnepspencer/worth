//! basis drift evidence for inherited retained state.
use super::*;
#[test]
fn a_fork_cannot_reuse_state_from_another_owner_installation() {
    let _guard = checkpoint_recovery_test_guard();
    let model = prefix::model();
    let kept =
        installation::install_variant::<false, TOTALS_WORK, 1, 0>(None, Default::default(), |g| {
            model.seed(g)
        });
    let fresh =
        installation::install_variant::<false, TOTALS_WORK, 1, 3>(None, Default::default(), |g| {
            model.seed(g)
        });
    judge(
        &run(&kept, kept.current_world()),
        &run(&fresh, fresh.current_world()),
        None,
    );
    let child = fork(&kept, kept.current_world());
    let full_child = fork(&fresh, fresh.current_world());
    let edit = at_demand_scope(EntryEdit::new(
        "even",
        7,
        super::super::super::entry_edit::EntryFact::Value,
        2.5_f64.to_bits(),
    ));
    apply(&kept, child, vec![Change::Entry(edit.clone())], &mut 0x6119);
    apply(&fresh, full_child, vec![Change::Entry(edit)], &mut 0x6119);
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            REINSTALL.with(|value| value.set(false));
        }
    }
    let _reset = Reset;
    REINSTALL.with(|value| value.set(true));
    judge(
        &run(&kept, child),
        &run(&fresh, full_child),
        Some(Run::Full(Cause::OtherInstallation)),
    );
}
