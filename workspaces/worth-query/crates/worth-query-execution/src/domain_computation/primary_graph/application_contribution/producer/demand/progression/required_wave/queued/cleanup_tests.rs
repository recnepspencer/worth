//! An already drained queue still enters the paid cleanup door.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;
use worth_relational::facade::mvcc::CompanionPreflightBudget;
fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(work),
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}
fn drained_cleanup(one_short: bool) {
    with_committed_own_write(|world, receipt, _, _| {
        let runtime = &world.application;
        let registry = &runtime.output_demands;
        let (anchor, ready) = registry.fixture_refusing_ready();
        // Fixture publication opens its own host calls before this payer opens.
        let baseline = registry.fixture_released_publication_cleanup(receipt);
        let pending = registry.fixture_cleanup_head();
        let before = registry.fixture_reserved_balances();
        world
            .application
            .with_host_advancement(|phase| {
                let execution = phase
                    .execution_request_for(&world.application.product_runtime)
                    .unwrap();
                let mut selection = runtime.demand_request_admission();
                let branch = runtime.current_world();
                let (shared, positioned) = super::super::selection::select_required_basis(
                    &execution,
                    runtime,
                    branch,
                    &mut selection,
                )
                .unwrap();
                let head = runtime
                    .output_demands
                    .publication_head_selection(
                        shared
                            .selected()
                            .product()
                            .observation()
                            .lifecycle_incarnation(),
                        &mut selection,
                    )
                    .unwrap();
                let wave = RequiredWaveSelection::from_basis(
                    shared,
                    positioned,
                    ready,
                    branch,
                    RequiredWaveTarget::Caller,
                    head,
                );
                {
                    assert_ne!(
                        before, baseline,
                        "the released publication really retains custody"
                    );
                    let mut queue = QueuedRequiredRoots::new();
                    queue.drained = true;
                    if one_short {
                        // The door declares branch bytes + four comparisons and two owner units.
                        // The drain pays two entry units, then funds a 64-byte terminal subject
                        // and its String header before it can detach its first pending row.
                        let door = wave.positioned.branch_id().0.len() as u64 + 4 + 2;
                        let first_detach_minimum = 2 + 64 + std::mem::size_of::<String>() as u64;
                        let mut short = admission(door + first_detach_minimum - 1);
                        let failure = queue
                            .next_claim(runtime, &wave, &mut short)
                            .err()
                            .expect("the first cleanup step is one unit short");
                        assert_eq!(
                            failure.kind(),
                            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                        );
                        assert_eq!(
                            short.charged_work(),
                            door + 2,
                            "the refused terminal increment is not charged"
                        );
                        assert_eq!(
                            registry.fixture_cleanup_head(),
                            pending,
                            "the unpaid remainder stays noted by identity"
                        );
                        assert_eq!(
                            registry.fixture_reserved_balances(),
                            before,
                            "no custody is destroyed on the refused step"
                        );
                    }
                    // This refund proof funds the entire finite drain in one admission.
                    // The one-short case and every-unit drain table pin refusal charges.
                    assert!(queue
                        .next_claim(runtime, &wave, &mut admission(u64::MAX))
                        .unwrap()
                        .is_none());
                    assert_eq!(
                        registry.fixture_reserved_balances(),
                        baseline,
                        "one paying drained-door admission restores all three totals"
                    );
                    assert_eq!(registry.fixture_cleanup_head(), None);
                }
                drop(wave);
                drop(anchor);
            })
            .expect("the owner proof owns its request");
    });
}
#[test]
fn a_drained_queue_pays_for_released_publication_custody() {
    drained_cleanup(false);
}
#[test]
fn a_refused_drained_cleanup_leaves_its_remainder_for_the_next_admission() {
    drained_cleanup(true);
}
