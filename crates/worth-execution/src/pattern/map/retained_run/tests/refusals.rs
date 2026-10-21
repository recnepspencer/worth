use super::*;

#[test]
fn context_fallback_and_invalid_membership_refuse_before_effect_and_preserve_prior() {
    let memory = SerialMemoryBudget::new(Some(1_000_000));
    let request = serial(&memory);
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(entries([1, 2])).unwrap();
    let (prior, _) = complete(
        map.run_retained_run(
            ExecutionRequest::serial(&request),
            ExecutionWorkCeiling::new(Some(100)),
            |_, context| {
                Ok::<_, MapKernelFailure<()>>(
                    context
                        .remaining_work()
                        .expect("fixture installs finite work"),
                )
            },
        )
        .unwrap(),
    );
    let effects = AtomicUsize::new(0);
    let cause = prior.advance(
        ExecutionRequest::serial(&request),
        ExecutionWorkCeiling::new(Some(100)),
        entries([2]),
        empty_ids(),
        |v, context| {
            effects.fetch_add(1, Ordering::SeqCst);
            kernel(v, context)
        },
    );
    assert!(
        matches!(cause, Err(RetainedRunDenial::Native(RetainedMapDenial::RequiresExecution(id))) if id == PartitionIdentity::new(1))
    );
    assert_eq!(effects.load(Ordering::SeqCst), 0);
    assert_eq!(prior.partition_count(), 2);
    // Replacing the unqualified slot by real evaluation is a lawful positive.
    let (fresh, _) = complete(
        prior
            .advance(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(100)),
                entries([1, 2]),
                empty_ids(),
                kernel,
            )
            .unwrap(),
    );
    assert!(fresh.requires_execution().is_none());
    let removed =
        CanonicalUniqueVec::try_from_sorted_unique(vec![PartitionIdentity::new(99)]).unwrap();
    assert!(matches!(
        fresh.advance(
            ExecutionRequest::serial(&request),
            ExecutionWorkCeiling::new(Some(100)),
            entries([1]),
            removed,
            kernel
        ),
        Err(RetainedRunDenial::UnknownRemoval(_))
    ));
    assert_eq!(
        *fresh.result(PartitionIdentity::new(1)).unwrap().value(),
        10
    );
}

#[test]
fn tight_new_node_funding_preserves_original_and_clone_until_final_drop() {
    let memory = SerialMemoryBudget::new(Some(1_000_000));
    let request = serial(&memory);
    let prior = seed(10, &request);
    let held = memory.charged_memory_bytes();
    let blocker = memory
        .reserve(memory.limit().expect("fixture installs finite memory") - held - 128)
        .unwrap();
    let calls = AtomicUsize::new(0);
    let refused = prior.advance(
        ExecutionRequest::serial(&request),
        ExecutionWorkCeiling::new(Some(100)),
        entries([5]),
        empty_ids(),
        |v, context| {
            calls.fetch_add(1, Ordering::SeqCst);
            kernel(v, context)
        },
    );
    assert!(refused.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        *prior.result(PartitionIdentity::new(5)).unwrap().value(),
        50
    );
    drop(blocker);
    assert_eq!(memory.charged_memory_bytes(), held);
    let (next, _) = complete(
        prior
            .advance(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(100)),
                entries([5]),
                empty_ids(),
                kernel,
            )
            .unwrap(),
    );
    let clone = next.clone();
    drop(next);
    drop(prior);
    assert!(memory.charged_memory_bytes() > 0);
    assert_eq!(
        *clone.result(PartitionIdentity::new(1)).unwrap().value(),
        10
    );
    drop(clone);
    drop(request);
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn cancelled_changed_suffix_remains_request_owned_and_fresh_retry_succeeds() {
    let memory = SerialMemoryBudget::new(Some(1_000_000));
    let request = serial(&memory);
    let prior = seed(4, &request);
    let cancellation = crate::CancellationSource::new();
    let cancelled = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
    let result = prior.advance(
        ExecutionRequest::serial(&cancelled),
        ExecutionWorkCeiling::new(Some(100)),
        entries([1, 4]),
        empty_ids(),
        |value, context| {
            context.checkpoint(1)?;
            if *value == 4 {
                cancellation.cancel();
                let _ = context.checkpoint(1);
            }
            Err::<u64, _>(MapKernelFailure::Domain(()))
        },
    );
    assert!(matches!(
        result,
        Err(RetainedRunDenial::Native(RetainedMapDenial::Request(
            crate::WorkCeilingDenial::Stopped(crate::MapKernelStop::Cancelled)
        )))
    ));
    let (next, _) = complete(
        prior
            .advance(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(100)),
                entries([1, 4]),
                empty_ids(),
                kernel,
            )
            .unwrap(),
    );
    assert_eq!((next.partition_count(), next.charged_work()), (4, 20));
}
