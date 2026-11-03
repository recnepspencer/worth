//! Every rejected roster reports its true native subject and leaves the head unchanged.
use super::*;
use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;

#[allow(clippy::too_many_arguments)]
pub(super) fn check(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    anchor: EntityId,
    roles: &[RequiredOutputRoleTarget],
    ordinary: EntityId,
    foreign: EntityId,
    independent: EntityId,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refuse = |selected: EntityId, roster: &[RequiredOutputRoleTarget], cause, subject| {
        let error = runtime
            .readmit_authored_source_retirement(execution, installed, &basis, selected, roster)
            .unwrap_err();
        let TransactionCommitError::Publication { error, .. } = error else {
            panic!("expected typed publication refusal: {error:?}")
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(cause),
            "{error:?}"
        );
        assert_eq!(error.context.affected_records, [RecordRef::Entity(subject)]);
        assert_eq!(
            runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .1
                .descriptor(),
            basis.descriptor()
        );
    };
    let last = match roles.last().unwrap().target {
        EntityReference::Existing(id) => id,
        _ => unreachable!(),
    };
    refuse(
        anchor,
        &roles[..roles.len() - 1],
        Cause::MissingMember,
        last,
    );
    let first_publication = match roles[1].target {
        EntityReference::Existing(id) => id,
        _ => unreachable!(),
    };
    let missing_publication = roles[1..3]
        .iter()
        .map(|role| match role.target {
            EntityReference::Existing(id) => id,
            _ => unreachable!(),
        })
        .min()
        .unwrap();
    refuse(
        anchor,
        &roles[..1],
        Cause::MissingPublication,
        missing_publication,
    );
    let mut mixed_postures = roles.to_vec();
    mixed_postures[1].posture = RequiredOutputRolePosture::Preserve;
    refuse(
        anchor,
        &mixed_postures,
        Cause::ExtraEntity,
        first_publication,
    );
    let mut repeated = roles.to_vec();
    repeated.push(roles[1].clone());
    refuse(anchor, &repeated, Cause::RepeatedTarget, first_publication);
    for (extra, cause) in [
        (ordinary, Cause::ExtraEntity),
        (foreign, Cause::DifferentSourcePublication),
        (independent, Cause::UnrelatedPublication),
    ] {
        let mut mixed = roles.to_vec();
        mixed.push(RequiredOutputRoleTarget {
            target: EntityReference::Existing(extra),
            ..roles[0].clone()
        });
        refuse(anchor, &mixed, cause, extra);
    }
    refuse(
        first_publication,
        roles,
        Cause::InvalidAuthoredAnchor,
        first_publication,
    );
    // The closure walk must use the original caller's charged work, never a fresh allowance.
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(8 * 1024 * 1024),
            Some(1),
        ),
    );
    let limited = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let error = runtime
        .readmit_authored_source_retirement(
            &ExecutionRequest::serial(&limited),
            installed,
            &basis,
            anchor,
            roles,
        )
        .unwrap_err();
    assert!(
        matches!(error, TransactionCommitError::Execution { denial, .. }
        if matches!(denial.kind, CommitExecutionDenialKind::Cause(
            RelationalExecutionDenialCause::WorkExhausted)))
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
