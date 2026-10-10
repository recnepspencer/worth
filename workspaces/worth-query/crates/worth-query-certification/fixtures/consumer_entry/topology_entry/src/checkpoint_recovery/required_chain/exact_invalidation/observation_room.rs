//! A branch that admits few product observations makes room for its callers
//! from the observations nothing holds.

use super::*;

/// A caller that only writes cannot fill a branch for good. A performed write
/// keeps the observation its source was read at for its required outputs,
/// and no later write replaces it: every body here is a root of its own.
/// More such writes than the branch admits observations are all performed.
/// Every output is then demanded and settles on the model in one advance,
/// and a root is written and demanded once more.
fn a_write_only_caller_leaves_room(starts_outputs: bool) {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_observing(None, ring_world::seed::<3>, Retained::AMPLE, 16);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f00);
    let at = "a write-only caller";
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    for index in 3..8 {
        court.create_ring(index, at);
        rings.push(Ring::created(index));
    }
    Reading::decisions();
    let write = |body: &str, y| {
        if starts_outputs {
            court.write_y_and_abandon_its_outputs(body, y, at);
        } else {
            court.write_y(body, y, at);
        }
    };
    for ring in &mut rings {
        (ring.a_y, ring.successor_y, ring.far_y) = (2, 2, 11);
        write(&ring.key("a"), ring.a_y);
        write(&ring.successor, ring.successor_y);
        write(&ring.key("source-c"), ring.far_y);
    }
    let at = "after a write-only caller";
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, at);
    }
    rings[1].a_y = 5;
    court.write_y(&rings[1].key("a"), 5, at);
    court.demand_ring(&mut rings, 1, at);
}

/// A performed write whose source the branch released for later callers is
/// no longer re-entered through its receipt: the recovery says the retained
/// basis is gone, the same request is answered as already committed and not
/// performed again, and the outputs are produced by their next demand.
#[test]
fn a_released_write_refuses_receipt_recovery_and_is_not_performed_again() {
    use worth_query_host::facade::application_entry::{
        WorthQueryApplicationPerformedMutationOutcome as Performed,
        WorthQueryRequiredOutputPreparationDenial as Preparation,
    };
    let _guard = checkpoint_recovery_test_guard();
    let application = install_observing(None, ring_world::seed::<3>, Retained::AMPLE, 16);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f80);
    let at = "a released write";
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    for index in 3..8 {
        court.create_ring(index, at);
        rings.push(Ring::created(index));
    }
    Reading::decisions();
    let body = rings[0].key("a");
    let source = request
        .query(PlanarRead {
            body_key: body.clone(),
        })
        .execute()
        .expect("the branch admits a courtroom read")
        .observed_sources()[0]
        .clone();
    let key = court.next_idempotency();
    let write = || {
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: body.clone(),
                replacement_y: length(2),
            })
            .expect_source(source.clone())
            .idempotency(&key)
            .execute_performed::<program::ChainProgram, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
    };
    let recover = |receipt| {
        request.recover_required_outputs::<program::ChainProgram, program::ChainRoot>(
            &application,
            receipt,
            PlanarOutputDemand::new(body.clone()),
            WorthQueryOutputDemandControls::host_policy(),
        )
    };
    let receipt = match write() {
        Ok(Performed::Performed(performed)) => performed.receipt().clone(),
        _ => panic!("{at}: the Y of {body} is written"),
    };
    rings[0].a_y = 2;
    assert!(
        recover(&receipt).is_ok(),
        "{at}: a held source is re-entered through its receipt"
    );

    // More writes than the branch admits observations: the first one's
    // source is released to make room.
    for ring in &mut rings {
        (ring.successor_y, ring.far_y) = (2, 11);
        court.write_y(&ring.successor, ring.successor_y, at);
        court.write_y(&ring.key("source-c"), ring.far_y, at);
        if ring.index > 0 {
            ring.a_y = 2;
            court.write_y(&ring.key("a"), ring.a_y, at);
        }
    }
    match recover(&receipt).err() {
        Some(Preparation::Demand(WorthQueryApplicationOutputDemandDenial::Demand(denial)))
            if denial.kind()
                == primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable => {}
        other => panic!("{at}: a released source has no retained basis: {other:?}"),
    }
    match write() {
        Ok(Performed::NotPerformed(WorthQueryApplicationMutationOutcome::AlreadyCommitted(
            replayed,
        ))) => assert!(
            replayed.is_same_authoritative_commit(&receipt),
            "{at}: the same request answers the commit it performed"
        ),
        _ => panic!("{at}: a released write is not performed again"),
    }
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, at);
    }
}

/// The caller drops each write: its required outputs never start.
#[test]
fn writes_whose_outputs_never_start_leave_room_for_a_demand_and_another_write() {
    a_write_only_caller_leaves_room(false);
}

/// The caller starts each write's required outputs and drops them before
/// they advance. Each output keeps its obligation and is produced from the
/// source its next demand discloses.
#[test]
fn writes_whose_started_outputs_never_advance_leave_room_for_a_demand_and_another_write() {
    a_write_only_caller_leaves_room(true);
}
