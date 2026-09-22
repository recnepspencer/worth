use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchSetAdoptionProgress,
    WorthQueryBranchSetAdoptionRecoveryOutcome,
};
use worth_query_host::facade::primary_graph::{
    RuntimeWorldRecoveryDenial, RuntimeWorldSettledRelationalAdoptionDenial,
    WorthQueryBranchAdoptionRecoveryDenial,
};

use super::{fork, target_revision, P1_ONLY_DIMENSION};
use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::DimensionProgramP1;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

#[test]
fn unpublished_middle_branch_recovers_without_stranding_prefix_or_suffix() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b, c], NonZeroUsize::new(3).unwrap())
        .unwrap();
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("all three branches preflight");

    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == a
    ));
    host.runtime().fail_next_durable_append_for_test();
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::ProductUnpublished { branch, .. } if *branch == b
    ));
    let recovery = match adoption.begin_recovery() {
        Ok(recovery) => recovery,
        Err(_) => panic!("the unpublished middle branch must transfer exact custody"),
    };
    assert_eq!(recovery.branch(), b);
    assert_eq!(recovery.progress().len(), 1);
    assert_eq!(recovery.pending_branches().collect::<Vec<_>>(), [c]);
    assert!(recovery
        .unpublished_custody()
        .is_some_and(|unpublished| unpublished.relational_requires_settlement()));

    host.runtime().fail_next_durable_append_for_test();
    let failure = host
        .runtime()
        .request(&principal, &scope)
        .recover_branch_set_adoption(recovery)
        .err()
        .expect("a second durability loss must retain the whole branch-set recovery");
    assert!(matches!(
        &failure,
        worth_query_host::facade::application_entry::WorthQueryBranchSetAdoptionRecoveryFailure::Recovery {
            failure,
            ..
        } if matches!(
            failure.denial(),
            WorthQueryBranchAdoptionRecoveryDenial::AdoptionPreparation(
                RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
                    RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable
                )
            )
        )
    ));
    let recovery = failure.into_recovery();
    assert_eq!(recovery.branch(), b);
    assert_eq!(recovery.progress().len(), 1);
    assert_eq!(recovery.pending_branches().collect::<Vec<_>>(), [c]);

    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .recover_branch_set_adoption(recovery);
    let mut adoption = match outcome {
        Ok(WorthQueryBranchSetAdoptionRecoveryOutcome::Performed { adoption, cleanup }) => {
            cleanup.expect("the superseded recovery record must drain");
            adoption
        }
        Ok(WorthQueryBranchSetAdoptionRecoveryOutcome::NoEffect { .. }) => {
            panic!("the exact recovery unexpectedly had no effect")
        }
        Ok(WorthQueryBranchSetAdoptionRecoveryOutcome::ProductUnpublished { .. }) => {
            panic!("the one-shot durability failure unexpectedly repeated")
        }
        Err(_) => panic!("the exact branch-set recovery must remain admitted"),
    };
    assert_eq!(adoption.progress().len(), 2);
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == c
    ));
    let closed = match adoption.close() {
        Ok(closed) => closed,
        Err(_) => panic!("recovered progress and its suffix must close"),
    };
    assert_eq!(closed.progress().len(), 3);

    let p1 = host.supported_program::<DimensionProgramP1>().unwrap();
    for (ordinal, branch) in [a, b, c].into_iter().enumerate() {
        assert_eq!(
            settle(set_dimension(
                &p1,
                branch,
                P1_ONLY_DIMENSION,
                0x9175_4040 + ordinal as u64
            )),
            DimensionVerdict::Performed(P1_ONLY_DIMENSION)
        );
    }
}

#[test]
fn unpublished_middle_branch_can_release_custody_without_relabeling_untouched_suffix() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = fork(&host, a);
    let c = fork(&host, a);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b, c], NonZeroUsize::new(3).unwrap())
        .unwrap();
    let target = target_revision(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut adoption = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b, c])
        .unwrap()
        .programs()
        .adopt(&target)
        .prepare(64)
        .expect("all three branches preflight");

    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { branch, .. } if *branch == a
    ));
    host.runtime().fail_next_durable_append_for_test();
    assert!(matches!(
        adoption.advance().unwrap().unwrap(),
        WorthQueryBranchSetAdoptionProgress::ProductUnpublished { branch, .. } if *branch == b
    ));
    let recovery = match adoption.begin_recovery() {
        Ok(recovery) => recovery,
        Err(_) => panic!("the unpublished middle branch must transfer exact custody"),
    };

    let cancellation = WorthQueryCancellationSource::new();
    let recovery_scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(30),
        cancellation.token(),
    );
    let recovery_principal = authenticate_operator(host.installed_schema(), &recovery_scope);
    let pause = host
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare(NonZeroUsize::new(1).unwrap());
    let outcome = std::thread::scope(|threads| {
        let recovering = threads.spawn(|| {
            host.runtime()
                .request(&recovery_principal, &recovery_scope)
                .recover_branch_set_adoption(recovery)
        });
        assert!(
            pause.wait_until_reached(Duration::from_secs(10)),
            "recovery must reach the bounded final product comparison"
        );
        cancellation.cancel();
        pause.release();
        recovering.join().expect("recovery must not panic")
    });
    let recovery = match outcome {
        Ok(WorthQueryBranchSetAdoptionRecoveryOutcome::ProductUnpublished {
            recovery,
            prior_cleanup,
        }) => {
            prior_cleanup.expect("the consumed prior recovery record must drain");
            recovery
        }
        _ => panic!("cancellation after the settled effect must retain releasable custody"),
    };

    let (cancelled, cleanup) = match host
        .runtime()
        .request(&principal, &scope)
        .release_branch_set_adoption_recovery(recovery, 0)
    {
        Ok(released) => released,
        Err(
            worth_query_host::facade::application_entry::WorthQueryBranchSetAdoptionRecoveryReleaseFailure::Recovery {
                denial,
                ..
            },
        ) => panic!("release unexpectedly retained recovery: {denial:?}"),
        Err(
            worth_query_host::facade::application_entry::WorthQueryBranchSetAdoptionRecoveryReleaseFailure::OwnerCleanup {
                failure,
                ..
            },
        ) => panic!("release unexpectedly retained owner cleanup: {:?}", failure.denial()),
    };
    assert!(cleanup.is_complete());
    assert_eq!(cancelled.progress().len(), 1);
    assert_eq!(cancelled.cancelled_branch_count(), 2);

    let p1 = host.supported_program::<DimensionProgramP1>().unwrap();
    assert_eq!(
        settle(set_dimension(&p1, a, P1_ONLY_DIMENSION, 0x9175_4050)),
        DimensionVerdict::Performed(P1_ONLY_DIMENSION),
        "release cannot erase the already-performed prefix"
    );
    assert_eq!(
        settle(set_dimension(
            &host,
            c,
            super::P0_ONLY_DIMENSION,
            0x9175_4051
        )),
        DimensionVerdict::Performed(super::P0_ONLY_DIMENSION),
        "release cannot relabel the untouched suffix"
    );
}
