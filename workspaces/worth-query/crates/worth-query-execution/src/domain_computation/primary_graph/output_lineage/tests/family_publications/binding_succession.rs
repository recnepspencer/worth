//! Descriptive locator tests; these records grant no live identity or authority.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;

fn scope(court: &Court) -> WorthQueryOperationScopeBinding {
    let entity = court.source.scope;
    WorthQueryOperationScopeBinding::axis_probe_scope(
        court.source.runtime_authority,
        court.source.schema.clone(),
        "index selection",
        entity.partition_id(),
        entity.local_slot(),
        entity.generation(),
        entity.partition_id(),
        entity.local_slot(),
        entity.generation(),
    )
}

fn resolve(court: &Court) -> Result<WorthQueryPriorOutputBindingResolution, Stop> {
    court
        .lineage
        .resolve_binding_succession::<Preserve, Initial>(
            &worth_execution::ExecutionRequest::serial(&court.request),
            &scope(court),
            court.coordinate.occurrence,
            court.coordinate.generation,
            [1; 32],
        )
}

#[test]
fn paired_prior_tracks_publication_not_old_binding_or_inherited_settlement() {
    let mut court = Court::new();
    court.publish::<Initial>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    let preserved = court.publish::<Preserve>(
        3,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    let reissued = court.publish::<Initial>(
        5,
        Some(1),
        4,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.inherit_settlement::<Initial, Initial>(5, 2);
    court.publish::<Preserve>(
        7,
        Some(2),
        6,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    assert!(Arc::ptr_eq(
        &resolve(&court).unwrap().correspondence.unwrap(),
        &reissued
    ));
    court.coordinate.generation = 4;
    assert!(Arc::ptr_eq(
        &resolve(&court).unwrap().correspondence.unwrap(),
        &preserved
    ));
    let ancestor = court.coordinate;
    court.lineage.origins.insert(court.child, ancestor);
    court.coordinate = ProductCoordinate {
        occurrence: court.child,
        generation: 2,
    };
    let child = court.publish::<Initial>(
        1,
        Some(1),
        8,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    assert!(Arc::ptr_eq(
        &resolve(&court).unwrap().correspondence.unwrap(),
        &child
    ));
    court.coordinate.generation = 0;
    assert!(Arc::ptr_eq(
        &resolve(&court).unwrap().correspondence.unwrap(),
        &preserved
    ));
    court.lineage.origins.remove(&court.child);
    assert!(
        resolve(&court).unwrap().correspondence.is_none(),
        "unselected ancestry is absent"
    );
}

#[test]
fn paired_prior_refuses_ambiguous_foreign_and_cancelled_selection() {
    let mut court = Court::new();
    assert!(resolve(&court).unwrap().correspondence.is_none());
    court.publish::<Initial>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.publish::<Preserve>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    for _ in 0..2 {
        assert!(matches!(resolve(&court), Err(Stop::SelectedSourceMismatch)));
        court
            .lineage
            .output_families
            .get_mut("family")
            .unwrap()
            .reverse();
    }
    let execution = &worth_execution::ExecutionRequest::serial(&court.request);
    assert!(
        matches!(
            court
                .lineage
                .resolve_binding_succession::<OtherRole, Initial>(
                    execution,
                    &scope(&court),
                    court.coordinate.occurrence,
                    court.coordinate.generation,
                    [1; 32],
                ),
            Err(Stop::SelectedSourceMismatch)
        ),
        "foreign installed bindings cannot form a pair"
    );
    let cancellation = worth_execution::CancellationSource::new();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    cancellation.cancel();
    assert!(matches!(
        court
            .lineage
            .resolve_binding_succession::<Preserve, Initial>(
                &worth_execution::ExecutionRequest::serial(&request),
                &scope(&court),
                court.coordinate.occurrence,
                court.coordinate.generation,
                [1; 32],
            ),
        Err(Stop::Execution(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled
        ))
    ));
}
