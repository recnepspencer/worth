//! Another writer supersedes the caller's actual publication between passes.
use super::super::super::entry_edit::EntryFact;
use super::super::super::region_output::{arm_own_write, OwnWrite, OwnWriteRead};
use super::super::differential::alphabet::Lcg;
use super::*;
use std::sync::Arc;
#[test]
fn another_writer_between_passes_leaves_pending_after_one_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let model = Model::new(&mut Lcg(SEEDS[0]));
    let app = Arc::new(install(|graph| model.seed(graph)));
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    edit(
        &request,
        &app,
        EntryEdit::new("", 0, EntryFact::Value, 4_f64.to_bits()),
        612_001,
    );
    let scheduled = Arc::new(AtomicUsize::new(0));
    let writer = Arc::clone(&app);
    let recorded = Arc::clone(&scheduled);
    app.interleave_before_owned_reobservation_for_test(move || {
        let (scope, principal) = authenticate(&writer);
        edit(
            &writer.request(&principal, &scope),
            &writer,
            EntryEdit::new("", 2, EntryFact::Value, 6_f64.to_bits()),
            612_002,
        );
        recorded.fetch_add(1, Ordering::SeqCst);
    });
    arm_own_write(Some(OwnWrite {
        number: 1,
        bits: 5_f64.to_bits(),
        read: OwnWriteRead::Observed,
    }));
    room().clear();
    let mut output = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<false, TOTALS_WORK, 1, 0>, RegionConnection>(
            &app,
        )
        .unwrap();
    let advanced = output.advance(&request).unwrap();
    arm_own_write(None);
    assert!(matches!(
        advanced,
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    assert_eq!(
        scheduled.load(Ordering::SeqCst),
        1,
        "the second writer committed between passes"
    );
    assert_eq!(room().len(), 1, "the advance enters the handler only once");
    assert!(
        matches!(
            output.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ),
        "the next advance settles over both writes"
    );
}
