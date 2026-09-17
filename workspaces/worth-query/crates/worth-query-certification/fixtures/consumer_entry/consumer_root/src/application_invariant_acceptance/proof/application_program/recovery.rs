use std::num::NonZeroUsize;

use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation, PlanarVertex};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarEdit, PlanarMutation, PlanarOutputDemand, PlanarOutputRead, PlanarRead,
    PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

mod settlement;
mod snapshot_pressure;
mod supersession;
pub(super) use settlement::settle_recovered;
pub(super) use snapshot_pressure::snapshot_pressure_preserves_recoverable_source;
pub(super) use supersession::superseded_completion_is_terminal;

pub(super) fn caller_disposal_before_progress_recovers(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable")
        .observed_sources()[0]
        .clone();
    let controls = output_controls();
    let intent = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    let outcome = request
        .mutate(intent.clone())
        .expect_source(source.clone())
        .idempotency(&10_002)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("source publication admits its required output");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source publication must be fresh")
    };
    let source_receipt = performed.receipt().clone();
    drop(performed);
    let replay = request
        .mutate(intent)
        .expect_source(source)
        .idempotency(&10_002)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("the retry reaches the committed source");
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(replay) = replay else {
        panic!("the source retry must not perform twice")
    };
    let replay_receipt = replay
        .receipt()
        .expect("an idempotent retry retains its committed receipt");
    assert_eq!(
        replay_receipt.commit_reference(),
        source_receipt.commit_reference()
    );
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1,
        "caller disposal preserves the exact source carrier"
    );
    let truncated_recovery = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerTruncatedProgramRoot>(
            &world.application,
            replay_receipt,
            PlanarOutputDemand::new("anchor-b"),
            output_controls(),
        );
    let Err(denial) = truncated_recovery else {
        panic!("a truncated root cannot adopt retained source custody")
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::UndeclaredOutputRoot
    ));
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1,
        "root admission denial preserves usable retained custody"
    );
    let ordinary = request
        .demand(PlanarOutputDemand::new("anchor-b"))
        .controls(output_controls())
        .start()
        .expect("an ordinary interest may coexist with prepared program custody");
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1,
        "ordinary demand admission cannot consume required program custody"
    );
    let unrelated_source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the unrelated source is readable on the same branch")
        .observed_sources()[0]
        .clone();
    request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::CreateCycle(vec![
                PlanarVertex {
                    body_key: "other-a".to_owned(),
                    x: length(1),
                    y: length(1),
                },
                PlanarVertex {
                    body_key: "other-b".to_owned(),
                    x: length(2),
                    y: length(1),
                },
                PlanarVertex {
                    body_key: "other-c".to_owned(),
                    x: length(2),
                    y: length(3),
                },
            ]),
            validator_work: 4_096,
        }))
        .expect_source(unrelated_source)
        .idempotency(&10_012)
        .execute_in_program(&world.application)
        .expect("an intervening ordinary commit lands on the same branch");
    let recovered_once = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            replay_receipt,
            PlanarOutputDemand::new("anchor-b"),
            controls,
        )
        .expect("the installed program re-enters its owner-retained obligation");
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0,
        "recovery adopts the retained carrier across a newer same-branch head"
    );
    drop(recovered_once);
    let mut recovered = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &source_receipt,
            PlanarOutputDemand::new("anchor-b"),
            controls,
        )
        .expect("interrupted recovery re-enters the same receipt-bound obligation");
    let settled = loop {
        match recovered.advance(&request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    let row = request
        .at(settled.observation())
        .query(PlanarOutputRead {
            body_key: "final:anchor-b".to_owned(),
        })
        .execute()
        .expect("the recovered transitive result remains readable");
    assert_eq!(row.rows()[0].value, length(4));
    let completed = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &source_receipt,
            PlanarOutputDemand::new("anchor-b"),
            controls,
        )
        .err()
        .expect("settled output cannot be recovered as pending custody");
    assert_eq!(
        completed.recovery_posture(),
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal,
    );
    drop(ordinary);
}

pub(super) fn changed_root_cannot_adopt_stale_prepared_source(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_018)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("the source publication prepares its installed program");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source publication must be fresh")
    };
    let source_receipt = performed.receipt().clone();
    drop(performed);
    let changed_source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the published source is readable")
        .observed_sources()[0]
        .clone();
    let changed = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(5),
        })
        .expect_source(changed_source)
        .idempotency(&10_019)
        .execute_performed(&world.application)
        .expect("a distinct program source action changes the same occurrence");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(changed) = changed else {
        panic!("the distinct source action must publish a fresh revision")
    };
    let changed_receipt = changed.receipt().clone();
    drop(changed);

    let denial = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &source_receipt,
            PlanarOutputDemand::new("anchor-b"),
            output_controls(),
        )
        .err()
        .expect("a newer source revision cannot consume older prepared custody");
    assert_eq!(
        denial.recovery_posture(),
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal,
    );
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
    ) = denial
    else {
        panic!("changed root must preserve the exact output-demand denial")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded
    );
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1,
        "only the newer source retains prepared custody"
    );
    drop(
        request
            .recover_required_outputs::<crate::ConsumerProgram>(
                &world.application,
                &changed_receipt,
                PlanarOutputDemand::new("anchor-b"),
                output_controls(),
            )
            .expect("the newer source retains its own exact custody"),
    );
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0,
        "both old and new source custody are retired after exact recovery"
    );
}

pub(super) fn resource_denial_preserves_source_and_delivery(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source_result = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = source_result.observed_sources()[0].clone();
    let denied_controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    );
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-c".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_003)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("the source operation reaches its installed program");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source publication must succeed before derived admission")
    };
    let failure = performed
        .start_required_outputs(&request, denied_controls)
        .err()
        .expect("insufficient derived resources deny required-output start");
    assert_eq!(
        failure.denial().recovery_posture(),
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Retryable,
    );
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
    ) = failure.denial()
    else {
        panic!("resource denial must preserve the exact output-demand cause")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(failure.result().changed_vertices, 1);
    let _committed_source = failure
        .receipt()
        .committed_product_publication()
        .composite_commit();
    let mut retried = failure
        .into_performed()
        .start_required_outputs(&request, output_controls())
        .unwrap_or_else(|failure| panic!("exact prepared retry starts: {:?}", failure.denial()));
    let published = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the successful source publication remains visible");
    assert_eq!(published.rows()[0].y, length(2));
    let settled = loop {
        match retried.required_output_mut().advance(&request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    let row = request
        .at(settled.observation())
        .query(PlanarOutputRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the recovered output is retained");
    assert_eq!(row.rows()[0].value, length(3));
}

fn output_controls() -> WorthQueryOutputDemandControls {
    WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
}
