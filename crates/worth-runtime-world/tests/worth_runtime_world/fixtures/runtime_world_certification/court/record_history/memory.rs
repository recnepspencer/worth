// Real history allocations must retain the original serial/ancestor charge.
pub(crate) use super::*;
pub(crate) use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial, LeaseDenial, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};

pub(crate) fn assert_retained_memory(
    court: &NativeHistoryCourt,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) {
    for inherited in [false, true] {
        let budget = SerialMemoryBudget::new(Some(1_000_000));
        let parent =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let worker = SerialRequest::from_policy(
            &worth_foundational::ExecutionRequestPolicy::for_workers(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                NonZeroUsize::MIN,
            ),
            CancellationSource::new().token(),
            None,
        );
        let held = if inherited {
            assert!(!ExecutionRequest::serial(&worker).has_accounting());
            ExecutionRequest::serial(&parent)
                .run_native(|_| {
                    retain_event(court, basis, &budget, ExecutionRequest::serial(&worker))
                })
                .unwrap()
                .0
        } else {
            retain_event(court, basis, &budget, ExecutionRequest::serial(&parent))
        };
        assert!(
            budget.charged_memory_bytes() > 0,
            "event owns the retained selection/source after request scope exit"
        );
        assert_eq!(held.record(), &RecordRef::Entity(court.entities[0]));
        assert_cancelled_array_prefix(&held, &budget);
        drop(held);
        assert_eq!(
            budget.charged_memory_bytes(),
            0,
            "last event releases all native backing"
        );
    }
    let budget = SerialMemoryBudget::new(Some(0));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    for array in [false, true] {
        assert_exact_refusal(court, basis, ExecutionRequest::serial(&host), array);
    }
    let worker = SerialRequest::from_policy(
        &worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        CancellationSource::new().token(),
        None,
    );
    for array in [false, true] {
        let saturated = SerialMemoryBudget::new(Some(1_000_000));
        let parent =
            SerialRequest::from_memory(saturated.clone(), CancellationSource::new().token(), None);
        let mut entered = false;
        let outcome = ExecutionRequest::serial(&parent).run_native(|_| {
            entered = true;
            let room = saturated.limit().unwrap() - saturated.charged_memory_bytes();
            let _occupied = saturated.reserve(room).unwrap();
            assert_eq!(saturated.charged_memory_bytes(), saturated.limit().unwrap());
            assert_exact_refusal(court, basis, ExecutionRequest::serial(&worker), array)
        });
        assert!(
            entered,
            "funded parent must invoke actual child: {outcome:?}"
        );
        assert!(
            matches!(
                outcome,
                Ok(_)
                    | Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
                    | Err(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
                        _
                    )))
            ),
            "exact original memory refusal survives handoff: {outcome:?}"
        );
        assert_eq!(saturated.charged_memory_bytes(), 0);
    }
    assert_eq!(budget.charged_memory_bytes(), 0);
}

pub(crate) fn assert_exact_refusal(
    court: &NativeHistoryCourt,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    execution: ExecutionRequest<'_, '_>,
    array: bool,
) {
    if array {
        assert!(matches!(
            ExecutionRequestArrayBuilder::<u64>::allocate(execution, 1),
            Err(ExecutionRequestArrayDenial::Request(
                WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(_))
            ))
        ));
    } else {
        assert!(matches!(
            court.runtime.retain_record_history_subject(
                execution,
                basis,
                RecordRef::Entity(court.entities[0])
            ),
            Err(RelationalRecordHistoryDenial::Request(
                WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(_))
            ))
        ));
    }
}

pub(crate) fn retain_event(
    court: &NativeHistoryCourt,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    budget: &SerialMemoryBudget,
    execution: ExecutionRequest<'_, '_>,
) -> RelationalRecordHistoryChange {
    let subject = court
        .runtime
        .retain_record_history_subject(execution, basis, RecordRef::Entity(court.entities[0]))
        .unwrap();
    let subject_charge = budget.charged_memory_bytes();
    assert!(subject_charge > 0);
    let selection = court
        .runtime
        .retain_record_history_selection(
            execution,
            std::slice::from_ref(&subject),
            &[court.contracts[0].key().clone()],
            RelationalRecordHistorySelectionLimits {
                maximum_subjects: NonZeroUsize::MIN,
                maximum_aspects: NonZeroUsize::MIN,
                maximum_owned_bytes: NonZeroUsize::new(16_384).unwrap(),
            },
        )
        .unwrap();
    assert!(budget.charged_memory_bytes() > subject_charge);
    let observation = basis.observation();
    let selected = court
        .runtime
        .select_exact_commit(&observation, observation.commit_id().unwrap())
        .into_outcome()
        .unwrap();
    let mut cursor = court
        .runtime
        .record_aspect_history_cursor_from_selection(execution, selected, &selection)
        .unwrap();
    let before_page = budget.charged_memory_bytes();
    let page = cursor
        .next_page(&court.runtime, execution, limits(1, 1))
        .unwrap();
    assert_eq!(
        budget.charged_memory_bytes() - before_page,
        page.owned_backing_bytes() as u64,
        "one fixed array has exactly one backing ticket"
    );
    let held = page
        .entries()
        .next()
        .expect("the actual first canonical entity occurrence")
        .clone();
    drop(page);
    assert_eq!(budget.charged_memory_bytes(), before_page);
    drop((cursor, selection, subject));
    held
}

pub(crate) fn assert_cancelled_array_prefix(
    event: &RelationalRecordHistoryChange,
    budget: &SerialMemoryBudget,
) {
    let before = budget.charged_memory_bytes();
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let mut building = ExecutionRequestArrayBuilder::allocate(execution, 2).unwrap();
    building.push(event.clone()).unwrap();
    cancellation.cancel();
    assert!(matches!(
        building.push(event.clone()),
        Err(ExecutionRequestArrayDenial::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        ))
    ));
    drop(building);
    assert_eq!(
        budget.charged_memory_bytes(),
        before,
        "initialized prefix and refused input release their shared event holds"
    );
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let mut building =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&host), 1).unwrap();
    building.push(event.clone()).unwrap();
    cancellation.cancel();
    assert!(matches!(
        building.seal(),
        Err(ExecutionRequestArrayDenial::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        budget.charged_memory_bytes(),
        before,
        "failed final handoff drops filled backing"
    );
}
