use super::*;
use crate::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial;
use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::content::denial::{RelationalContentDenial, RelationalContentRejection};
use crate::derived_relation::definition::DerivedRelationDenial;
use crate::execution::denial_cause::RelationalExecutionDenialCause;
use crate::lineage::data::naming_denial::LineageNameDenial;
use crate::projected_member_domain::definition::ProjectedMemberDomainDenial;
use crate::validation::data::custom_rule::refusal::CustomInvariantRefusal;
use DenialClass::*;
use Disposition::*;

report!(RelationalExecutionDenialCause, cause => {
    use RelationalExecutionDenialCause as Cause;
    match cause {
        Cause::Cancelled => (Cancelled, Final, "relational.execution.cancelled", "The request was cancelled."),
        Cause::DeadlineElapsed => (DeadlineElapsed, Final, "relational.execution.deadline", "The request deadline elapsed."),
        Cause::Content(cause) => content_description(cause),
        _ => {
            let (class, disposition) = match cause {
                Cause::StorageUnavailable(_) | Cause::HistoricalVisibility(_) => (Unavailable, Escalate),
                Cause::PolicyMemoryExhausted { .. } | Cause::ProcessMemoryExhausted { .. }
                | Cause::DeclaredMemoryExhausted { .. } | Cause::WorkExhausted
                | Cause::ResultCapacityExceeded | Cause::ScratchCapacityExceeded
                | Cause::AllocationFailed => (Exhausted, Final),
                Cause::UnrelatedNestedLease | Cause::NoActiveExecutionScope => (Authorization, Reauthorize),
                Cause::ProjectionContract(_) | Cause::WorkerLimitExceedsParent
                | Cause::MemoryLimitExceedsParent | Cause::WorkLimitExceedsParent
                | Cause::ExpectedIdentitiesNotCanonical | Cause::EquivalenceContractUnavailable
                | Cause::UncheckedCustomKernel => (InvalidInput, CorrectInput),
                Cause::NestedStopped => (Unavailable, Final),
                Cause::ContentInconsistency | Cause::ChargedBytesOverflow
                | Cause::WorkCounterOverflow | Cause::AccountingUnavailable
                | Cause::WorkerFailed | Cause::MemoryOverflow => (Defect, Escalate),
                Cause::Cancelled | Cause::DeadlineElapsed | Cause::Content(_) => unreachable!(),
            };
            (class, disposition, "relational.execution.denied", "Relational execution refused the request.")
        }
    }
});
display_error!(RelationalExecutionDenialCause, cause => cause.diagnostic_summary(); cause => match cause {
    RelationalExecutionDenialCause::Content(rejection) => Some(rejection),
    RelationalExecutionDenialCause::StorageUnavailable(unavailable) => Some(unavailable),
    RelationalExecutionDenialCause::HistoricalVisibility(denial) => Some(denial),
    RelationalExecutionDenialCause::ProjectionContract(denial) => Some(denial),
    _ => None,
});

report!(RelationalBranchBasisDenial, cause => {
    use RelationalBranchBasisDenial as Cause;
    let (class, disposition) = match cause {
        Cause::ForeignRuntime { .. } => (Authorization, Reauthorize),
        Cause::UnknownBranch(_) => (NotFound, CorrectInput),
        Cause::ArchivedBranch(_) | Cause::DeletingBranch(_) | Cause::MaterializationUnavailable
        | Cause::UnavailableRetainedTarget | Cause::OwnerUnavailable => (Unavailable, Final),
        Cause::Cancelled => (Cancelled, Final),
        Cause::TimedOut => (DeadlineElapsed, Final),
        Cause::StaleReferenceGeneration | Cause::WrongBranchLocalTruthVersion => (Conflict, Reauthorize),
        Cause::RetentionCapacityExhausted | Cause::RetentionIdentityExhausted
        | Cause::SnapshotIdentityExhausted => (Exhausted, Final),
        Cause::OwnerFailure => (Defect, Escalate),
        Cause::MalformedDescriptor | Cause::UnsupportedDescriptorVersion { .. }
        | Cause::EmptyCommittedTargetMismatch | Cause::WrongImmutableTarget
        | Cause::MixedAxis(_) => (InvalidInput, CorrectInput),
    };
    (class, disposition, "relational.branch.basis_denied", "Branch basis admission was refused.")
});
display_error!(RelationalBranchBasisDenial);

fn content_description(
    cause: &RelationalContentRejection,
) -> (DenialClass, Disposition, &'static str, &'static str) {
    use RelationalContentRejection as Cause;
    let (class, disposition) = match cause {
        Cause::MissingContent => (NotFound, CorrectInput),
        Cause::BackingUnavailable { .. } | Cause::HistoryNotResident => (Unavailable, Escalate),
        Cause::IntegrityMismatch | Cause::EncodingInconsistency | Cause::ArithmeticOverflow => {
            (Defect, Escalate)
        }
        Cause::ContentIdentityConflict | Cause::DuplicateContentFormat => (Conflict, CorrectInput),
        Cause::SizeLimit { .. } => (Exhausted, CorrectInput),
        Cause::FormatBindingMissing
        | Cause::FormatMismatch
        | Cause::UnsupportedVersion
        | Cause::InvalidFormat
        | Cause::NonCanonicalContent
        | Cause::InputTypeMismatch
        | Cause::UnreferencedAttachment { .. } => (InvalidInput, CorrectInput),
    };
    (
        class,
        disposition,
        "relational.content.denied",
        "Content admission or read was refused.",
    )
}
report!(RelationalContentDenial, cause => match cause {
    RelationalContentDenial::Execution(cause) => cause.report_description(),
    cause => match cause.into_execution_cause() {
        RelationalExecutionDenialCause::Content(rejection) => content_description(&rejection),
        _ => unreachable!("non-execution content denial retains its semantic rejection"),
    },
});
report!(RelationalContentRejection, cause => content_description(cause));
display_error!(RelationalContentRejection);

report!(LineageNameDenial, cause => {
    use LineageNameDenial as Cause;
    let (class, disposition) = match cause {
        Cause::GroupNaming { denial, .. } => (denial.denial_class(), denial.disposition()),
        Cause::DuplicateBirth { .. } => (Conflict, CorrectInput),
        Cause::UnknownScheme { .. } | Cause::UnknownParentIdentity { .. } => (NotFound, CorrectInput),
        Cause::BirthWithoutCreate { .. } | Cause::LineagePostureMismatch { .. }
        | Cause::MissingBirth { .. } => (Invariant, CorrectInput),
        Cause::InvalidDiscriminatorBasis { .. } | Cause::UndeclaredRole { .. }
        | Cause::MissingAuthoredOccurrenceKey { .. } | Cause::MissingSplitDiscriminator { .. }
        | Cause::DuplicateParentIdentity { .. } | Cause::InvalidMergeKeeper { .. } => (InvalidInput, CorrectInput),
    };
    (class, disposition, "relational.lineage.name_denied", "Persistent name admission was refused.")
});

report!(AggregateGroupNamingDenial, cause => {
    use AggregateGroupNamingDenial as Cause;
    let class = match cause {
        Cause::LiveNameCollision { .. } | Cause::DuplicatePriorIdentity { .. } => Conflict,
        Cause::UnknownGroup { .. } | Cause::MemberNotLive { .. } => NotFound,
        Cause::PriorIdentityMismatch { .. } | Cause::RetiredIdentityLive { .. }
        | Cause::ContinuityRuleChanged | Cause::OverlappingMember { .. } => Invariant,
        Cause::InvalidSchemeId { .. } | Cause::EmptyGroup | Cause::DuplicateMemberName { .. }
        | Cause::DuplicateCarrier { .. } | Cause::MissingAnchorMember { .. } | Cause::SelectionClaim => InvalidInput,
    };
    (class, CorrectInput, "relational.aggregate.name_denied", "Aggregate group naming was refused.")
});
display_error!(AggregateGroupNamingDenial);

report!(DerivedRelationDenial, cause => {
    use DerivedRelationDenial as Cause;
    let (class, disposition) = match cause {
        Cause::DuplicateDefinition(_) => (Conflict, CorrectInput),
        Cause::MissingDefinition(_) | Cause::RecordUnavailable => (NotFound, CorrectInput),
        Cause::ForeignDefinition | Cause::ForeignObservation | Cause::OutsideDeclaredAccess => (Authorization, Reauthorize),
        Cause::WorkBudgetExceeded | Cause::SourceProgramIdentityExhausted => (Exhausted, Final),
        Cause::DiscontinuousEndpoints | Cause::InvalidGraphLaws | Cause::InvalidEventChain
        | Cause::OrderedPredecessorMembershipRequired | Cause::OrderedPredecessorMustBeToOne => (RelationLaw, CorrectInput),
        Cause::InvalidIdentity | Cause::CyclicDefinitions | Cause::InvalidExpression
        | Cause::InvalidPhysicalSource(_) | Cause::InvalidMemberDomain | Cause::InvalidMemberCase => (InvalidInput, CorrectInput),
    };
    (class, disposition, "relational.derived_relation.denied", "Derived relation admission was refused.")
});
display_error!(DerivedRelationDenial);

report!(ProjectedMemberDomainDenial, cause => {
    use ProjectedMemberDomainDenial as Cause;
    let (class, disposition) = match cause {
        Cause::DuplicateDomain(_) => (Conflict, CorrectInput),
        Cause::ForeignDomain | Cause::ForeignCase => (Authorization, Reauthorize),
        Cause::InvalidIdentity | Cause::InvalidCarrier(_) | Cause::EmptyCases
        | Cause::NoncanonicalCases => (InvalidInput, CorrectInput),
    };
    (class, disposition, "relational.projected_member.domain_denied", "Projected member domain admission was refused.")
});
display_error!(ProjectedMemberDomainDenial);

report!(CustomInvariantRefusal, _cause => {
    (Invariant, CorrectInput, "relational.invariant.refusal", "An installed custom invariant refused the candidate.")
});
display_error!(CustomInvariantRefusal);

macro_rules! typed_leaf_error {
    ($type:ty, $summary:literal) => {
        display_error!($type, _value => $summary; _value => None);
    };
}
typed_leaf_error!(
    crate::storage::data::storage_unavailable::RelationalStorageUnavailable,
    "Relational storage was unavailable."
);
typed_leaf_error!(
    crate::visibility::snapshot_states::historical_denial::HistoricalVisibilityDenial,
    "Historical visibility was refused."
);
typed_leaf_error!(crate::visibility::materialization::read_records::projection::contracts::denial::RelationalProjectionContractDenial, "Projection contract admission was refused.");
