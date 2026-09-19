use bank_domain::{
    estate::{EstateCaseId, EstateCaseStatus, MandatoryReviewKind, MandatoryReviewStatus},
    model::BankPrincipalId,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, EstateCaseStatusField, EstateExecutor,
        LegalAuthority, LegalAuthorityEstate, LegalAuthorityHolder, LegalAuthorityIdentityField,
        LegalAuthorityRecognizedField, MandatoryReviewIdentityField, MandatoryReviewKindField,
        MandatoryReviewStatusField, Principal, PrincipalIdentityField, ReleaseEstateOperation,
        ReviewEstate, ReviewPrincipal,
    },
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryEntityResolutionDenial,
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial,
};

#[derive(Debug)]
pub enum BankEstateReleaseProjectionDenial {
    MissingEstateIdentity,
    EstateMismatch,
    MissingEstateStatus,
    EstateNotOpen,
    ExecutorMissing,
    ExecutorIdentityMissing,
    ExecutorIdentityMismatch,
    ExecutorRelationCardinality { observed: usize },
    RecognizedExecutorAuthorityMissing,
    LegalAuthorityIdentityMissing,
    LegalAuthorityIdentityMismatch,
    LegalAuthorityRecognitionMissing,
    LegalAuthorityHolderCardinality { observed: usize },
    LegalAuthorityHolderMismatch,
    LegalAuthorityEstateCardinality { observed: usize },
    LegalAuthorityEstateMismatch,
    ReleaseReviewMissing,
    ReleaseReviewWrongKind,
    ReleaseReviewIncomplete,
    ReviewIdentityMissing,
    ReviewPrincipalIdentityMissing,
    ReviewEstateCardinality { observed: usize },
    ReviewEstateMismatch,
    ReviewPrincipalCardinality { observed: usize },
    EntityResolution(crate::BankEntityResolutionDenial),
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
}

pub(crate) fn project_release_readiness(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    command: super::ReleaseCommand,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    project_estate(reader, estate, command.estate)?;
    project_executor(reader, estate, command)?;
    project_release_review(reader, estate, command.review)
}

fn project_estate(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    expected_estate: EstateCaseId,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    let observed = reader
        .decision_field(estate, EstateCaseIdentityField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::MissingEstateIdentity)?;
    if observed != expected_estate {
        return Err(BankEstateReleaseProjectionDenial::EstateMismatch);
    }
    let status = reader
        .decision_field(estate, EstateCaseStatusField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::MissingEstateStatus)?;
    if status != EstateCaseStatus::Open {
        return Err(BankEstateReleaseProjectionDenial::EstateNotOpen);
    }
    Ok(())
}

fn project_executor(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    command: super::ReleaseCommand,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    let executor = reader.resolve_entity(PrincipalIdentityField::reference(), command.executor)?;
    require_selected_executor_identity(reader, &executor, command.executor)?;
    let authority =
        reader.resolve_entity(LegalAuthorityIdentityField::reference(), command.authority)?;
    require_selected_authority_identity(reader, &authority, command.authority)?;
    let recognized = reader
        .decision_field(&authority, LegalAuthorityRecognizedField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::LegalAuthorityRecognitionMissing)?;
    if !recognized {
        return Err(BankEstateReleaseProjectionDenial::RecognizedExecutorAuthorityMissing);
    }
    require_exact_authority_holder(reader, &authority, &executor)?;
    require_exact_authority_estate(reader, &authority, estate)?;
    reader.require_decision_relation(EstateExecutor::reference(), &executor, estate)?;
    Ok(())
}

fn require_selected_executor_identity(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    executor: &WorthQueryInvariantEntityIdentity<BankSchema, Principal>,
    expected: BankPrincipalId,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    let observed = reader
        .decision_field(executor, PrincipalIdentityField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::ExecutorIdentityMissing)?;
    if observed != expected {
        return Err(BankEstateReleaseProjectionDenial::ExecutorIdentityMismatch);
    }
    Ok(())
}

fn require_selected_authority_identity(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, LegalAuthority>,
    expected: bank_domain::estate::LegalAuthorityId,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    let observed = reader
        .decision_field(authority, LegalAuthorityIdentityField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::LegalAuthorityIdentityMissing)?;
    if observed != expected {
        return Err(BankEstateReleaseProjectionDenial::LegalAuthorityIdentityMismatch);
    }
    Ok(())
}

fn require_exact_authority_holder(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, LegalAuthority>,
    executor: &WorthQueryInvariantEntityIdentity<BankSchema, Principal>,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    let holders = reader.decision_relations_from(LegalAuthorityHolder::reference(), authority)?;
    let [holder] = holders.as_slice() else {
        return Err(
            BankEstateReleaseProjectionDenial::LegalAuthorityHolderCardinality {
                observed: holders.len(),
            },
        );
    };
    if holder.to() != executor {
        return Err(BankEstateReleaseProjectionDenial::LegalAuthorityHolderMismatch);
    }
    Ok(())
}

fn require_exact_authority_estate(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, LegalAuthority>,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    let estates = reader.decision_relations_from(LegalAuthorityEstate::reference(), authority)?;
    let [related] = estates.as_slice() else {
        return Err(
            BankEstateReleaseProjectionDenial::LegalAuthorityEstateCardinality {
                observed: estates.len(),
            },
        );
    };
    if related.to() != estate {
        return Err(BankEstateReleaseProjectionDenial::LegalAuthorityEstateMismatch);
    }
    Ok(())
}

fn project_release_review(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ReleaseEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    expected_review: bank_domain::estate::MandatoryReviewId,
) -> Result<(), BankEstateReleaseProjectionDenial> {
    let review =
        reader.resolve_entity(MandatoryReviewIdentityField::reference(), expected_review)?;
    reader
        .decision_field(&review, MandatoryReviewIdentityField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::ReviewIdentityMissing)?;
    let kind = reader
        .decision_field(&review, MandatoryReviewKindField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::ReleaseReviewMissing)?;
    if kind != MandatoryReviewKind::EstateRelease {
        return Err(BankEstateReleaseProjectionDenial::ReleaseReviewWrongKind);
    }
    let status = reader
        .decision_field(&review, MandatoryReviewStatusField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::ReleaseReviewMissing)?;
    if status != MandatoryReviewStatus::Completed {
        return Err(BankEstateReleaseProjectionDenial::ReleaseReviewIncomplete);
    }
    let estates = reader.decision_relations_from(ReviewEstate::reference(), &review)?;
    let [related_estate] = estates.as_slice() else {
        return Err(BankEstateReleaseProjectionDenial::ReviewEstateCardinality {
            observed: estates.len(),
        });
    };
    if related_estate.to() != estate {
        return Err(BankEstateReleaseProjectionDenial::ReviewEstateMismatch);
    }
    let reviewers = reader.decision_relations_to(ReviewPrincipal::reference(), &review)?;
    let [reviewer] = reviewers.as_slice() else {
        return Err(
            BankEstateReleaseProjectionDenial::ReviewPrincipalCardinality {
                observed: reviewers.len(),
            },
        );
    };
    reader
        .decision_field(reviewer.from(), PrincipalIdentityField::reference())?
        .ok_or(BankEstateReleaseProjectionDenial::ReviewPrincipalIdentityMissing)?;
    Ok(())
}

impl From<WorthQueryEntityResolutionDenial> for BankEstateReleaseProjectionDenial {
    fn from(denial: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(crate::BankEntityResolutionDenial::from_query(denial.kind()))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankEstateReleaseProjectionDenial {
    fn from(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            denial.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial> for BankEstateReleaseProjectionDenial {
    fn from(denial: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            denial.kind(),
        ))
    }
}

impl std::fmt::Display for BankEstateReleaseProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for BankEstateReleaseProjectionDenial {}
