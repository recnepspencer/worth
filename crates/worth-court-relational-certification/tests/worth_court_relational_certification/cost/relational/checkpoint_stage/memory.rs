//! The capture's memory: its exact boundary and its refusals' custody.
use super::*;

/// The member roster's own block: its shared counts, entry array handle and
/// memory reservation.
const ROSTER_BLOCK_BYTES: usize = 136;

#[test]
fn empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage() {
    let _guard = crate::leased_preparation::ordinary_context::serial();
    // The boundary is the capture's peak, when its bytes are admitted: the
    // 2,486 external-manifest bytes + 400 bytes the one stage retains + what
    // the capture already holds beside them: the member roster the checkpoint
    // keeps for `members()`, its owner block plus one entry per external member.
    // A capture without commits retains no canonical publication.
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let checkpoint = runtime_with_rows(0)
        .durability_authority()
        .native_checkpoint_with_policy(&execution, Allocation::SystemAllocation, worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
        .unwrap();
    assert!(inline_members(checkpoint.bytes()).is_empty());
    let members = checkpoint.members().len();
    assert_eq!(members, 1);
    let roster = ROSTER_BLOCK_BYTES
        + members
            * std::mem::size_of::<worth_relational::facade::durability::RelationalCheckpointMember>(
            );
    drop(checkpoint);
    for leased in [false, true] {
        let (bytes, held) = memory_outcome(leased, 1 << 20, false).unwrap();
        assert_eq!(bytes, 2_486, "leased={leased}");
        assert_eq!(held, (2_486 + roster) as u64, "leased={leased}");
        let boundary = (2_486 + 400 + roster) as u64;
        let fits = memory_outcome(leased, boundary, false);
        assert!(matches!(fits, Ok((2_486, _))), "leased={leased}: {fits:?}");
        let short = memory_outcome(leased, boundary - 1, false);
        assert!(
            matches!(
                short,
                Err(Denial::Request(Cause::PolicyMemoryExhausted {
                    requested: 2_486,
                    admitted: 2_485,
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
            .native_checkpoint_with_policy(&execution, Allocation::SystemAllocation, worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None }),
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
