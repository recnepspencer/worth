//! The capture's memory: its exact boundary and its refusals' custody.
use super::*;

#[test]
fn empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage() {
    let _guard = crate::leased_preparation::ordinary_context::serial();
    // The boundary is the capture's peak, when its bytes are admitted: the
    // 2,645 checkpoint bytes + 400 bytes the one stage retains + what the
    // capture already holds beside them. That is the member roster the
    // checkpoint keeps for `members()`, charging structure read from an
    // accepted run, and the borrowed table the wire encodes its inline members
    // from, one (locator, bytes) entry per member.
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let checkpoint = runtime_with_rows(0)
        .durability_authority()
        .native_checkpoint(&execution, Allocation::SystemAllocation)
        .unwrap();
    let table = inline_members(checkpoint.bytes()).len()
        * std::mem::size_of::<(RelationalDurableMemberLocator, &[u8])>();
    drop(checkpoint);
    for leased in [false, true] {
        let (bytes, held) = memory_outcome(leased, 1 << 20, false).unwrap();
        assert_eq!(bytes, 2_645, "leased={leased}");
        let roster = held - 2_645;
        let boundary = 2_645 + 400 + roster + table as u64;
        let fits = memory_outcome(leased, boundary, false);
        assert!(matches!(fits, Ok((2_645, _))), "leased={leased}: {fits:?}");
        let short = memory_outcome(leased, boundary - 1, false);
        assert!(
            matches!(
                short,
                Err(Denial::Request(Cause::PolicyMemoryExhausted {
                    requested: 2_645,
                    admitted: 2_644,
                    ancestor: 0,
                }))
            ),
            "leased={leased}: {short:?}"
        );
        // Cancellation outranks the memory refusal at the first gate.
        for memory in [boundary - 1, boundary] {
            match (leased, memory_outcome(leased, memory, true)) {
                (false, Err(Denial::Request(Cause::Cancelled))) => {}
                (true, Err(Denial::Allocation(cause))) => assert_eq!(
                    cause.classify(),
                    ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Stopped(
                        MapKernelStop::Cancelled
                    ))
                ),
                (_, outcome) => panic!("leased={leased} memory={memory}: {outcome:?}"),
            }
        }
    }
}

#[test]
fn checkpoint_refusals_preserve_request_custody_and_publication() {
    let _guard = crate::leased_preparation::ordinary_context::serial();
    for leased in [false, true] {
        assert!(memory_outcome(leased, 1 << 20, false).unwrap().0 > 0);
        assert!(matches!(
            memory_outcome(leased, 0, false),
            Err(Denial::Request(Cause::PolicyMemoryExhausted { .. })) | Err(Denial::Allocation(_))
        ));
        match (leased, memory_outcome(leased, 0, true)) {
            (false, Err(Denial::Request(Cause::Cancelled))) => {}
            (true, Err(Denial::Allocation(cause))) => assert_eq!(
                cause.classify(),
                ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Stopped(
                    MapKernelStop::Cancelled
                ))
            ),
            (_, outcome) => panic!("cancellation must outrank memory refusal: {outcome:?}"),
        }
    }
    let request = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(0)),
        ),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let runtime = runtime_with_rows(0);
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    assert!(matches!(
        runtime
            .durability_authority()
            .native_checkpoint(&execution, Allocation::SystemAllocation),
        Err(Denial::Request(Cause::WorkExhausted))
    ));
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
}
