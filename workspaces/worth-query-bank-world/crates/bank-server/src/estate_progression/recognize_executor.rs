use bank_domain::{
    estate::{EstateAction, EstateCaseId, LegalAuthorityId},
    model::BankPrincipalId,
    proposals::BankIdempotencyKey,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, EstateExecutor, LegalAuthorityEstate,
        LegalAuthorityHolder, LegalAuthorityIdentityField, LegalAuthorityRecognizedField,
        PrincipalIdentityField, RecognizeEstateExecutorOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    primary_graph::{
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryEntityResolutionDenial,
        WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
        WorthQueryInvariantProjectionTraversalDenial,
    },
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

#[derive(Debug)]
pub enum BankExecutorRecognitionProjectionDenial {
    AuthorityNotRecognized,
    RelationCardinality {
        relation: &'static str,
        expected: usize,
        observed: usize,
    },
    MissingEstateIdentity,
    AuthorityEstateMismatch,
    MissingHolderIdentity,
    AuthorityHolderMismatch,
    AlreadyRecognizedExecutor,
    EntityResolution(crate::BankEntityResolutionDenial),
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
}

impl BankIdentityRuntime {
    pub fn recognize_estate_executor_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let EstateAction::RecognizeExecutor {
            estate,
            executor,
            authority,
        } = action
        else {
            return Err(BankEstateProgressionDenial::CommandInput(
                "RecognizeEstateExecutorOperation",
            ));
        };
        super::program_outcome::program_outcome(
            self.request(principal, request)
                .mutate(bank_domain::schema::RecognizeEstateExecutor::new(
                    estate, executor, authority,
                ))
                .idempotency(key)
                .execute_capability_in_selected_program(self.application_program()),
            "RecognizeEstateExecutorOperation",
            |denial| {
                denial
                    .downcast::<BankExecutorRecognitionProjectionDenial>()
                    .map(BankEstateProgressionDenial::ExecutorRecognitionProjection)
            },
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RecognitionCommand {
    pub(crate) estate: EstateCaseId,
    pub(crate) executor: BankPrincipalId,
    pub(crate) authority: LegalAuthorityId,
}

pub(crate) fn recognition_command(
    action: EstateAction,
) -> Result<RecognitionCommand, BankEstateProgressionDenial> {
    match action {
        EstateAction::RecognizeExecutor {
            estate,
            executor,
            authority,
        } => Ok(RecognitionCommand {
            estate,
            executor,
            authority,
        }),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "RecognizeEstateExecutorOperation",
        )),
    }
}

pub(crate) fn project_recognition(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        RecognizeEstateExecutorOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    command: RecognitionCommand,
) -> Result<(), BankExecutorRecognitionProjectionDenial> {
    let authority =
        reader.resolve_entity(LegalAuthorityIdentityField::reference(), command.authority)?;
    let recognized = reader
        .decision_field(&authority, LegalAuthorityRecognizedField::reference())?
        .unwrap_or(false);
    if !recognized {
        return Err(BankExecutorRecognitionProjectionDenial::AuthorityNotRecognized);
    }
    require_authority_estate(reader, &authority, command.estate)?;
    require_authority_holder(reader, &authority, command.executor)?;
    let executor = reader.resolve_entity(PrincipalIdentityField::reference(), command.executor)?;
    let existing = reader.decision_relations_from(EstateExecutor::reference(), &executor)?;
    if existing.iter().any(|relation| relation.to() == estate) {
        return Err(BankExecutorRecognitionProjectionDenial::AlreadyRecognizedExecutor);
    }
    Ok(())
}

fn require_authority_estate(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        RecognizeEstateExecutorOperation,
    >,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, bank_domain::schema::LegalAuthority>,
    expected: EstateCaseId,
) -> Result<(), BankExecutorRecognitionProjectionDenial> {
    let relations = reader.decision_relations_from(LegalAuthorityEstate::reference(), authority)?;
    let [relation] = relations.as_slice() else {
        return Err(
            BankExecutorRecognitionProjectionDenial::RelationCardinality {
                relation: "LegalAuthorityEstate",
                expected: 1,
                observed: relations.len(),
            },
        );
    };
    let observed = reader
        .decision_field(relation.to(), EstateCaseIdentityField::reference())?
        .ok_or(BankExecutorRecognitionProjectionDenial::MissingEstateIdentity)?;
    if observed != expected {
        return Err(BankExecutorRecognitionProjectionDenial::AuthorityEstateMismatch);
    }
    Ok(())
}

fn require_authority_holder(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        RecognizeEstateExecutorOperation,
    >,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, bank_domain::schema::LegalAuthority>,
    expected: BankPrincipalId,
) -> Result<(), BankExecutorRecognitionProjectionDenial> {
    let relations = reader.decision_relations_from(LegalAuthorityHolder::reference(), authority)?;
    let [relation] = relations.as_slice() else {
        return Err(
            BankExecutorRecognitionProjectionDenial::RelationCardinality {
                relation: "LegalAuthorityHolder",
                expected: 1,
                observed: relations.len(),
            },
        );
    };
    let observed = reader
        .decision_field(relation.to(), PrincipalIdentityField::reference())?
        .ok_or(BankExecutorRecognitionProjectionDenial::MissingHolderIdentity)?;
    if observed != expected {
        return Err(BankExecutorRecognitionProjectionDenial::AuthorityHolderMismatch);
    }
    Ok(())
}

impl From<WorthQueryEntityResolutionDenial> for BankExecutorRecognitionProjectionDenial {
    fn from(denial: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(crate::BankEntityResolutionDenial::from_query(denial.kind()))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankExecutorRecognitionProjectionDenial {
    fn from(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            denial.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial>
    for BankExecutorRecognitionProjectionDenial
{
    fn from(denial: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            denial.kind(),
        ))
    }
}

impl std::fmt::Display for BankExecutorRecognitionProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorityNotRecognized => write!(formatter, "legal authority is not recognized"),
            Self::RelationCardinality {
                relation,
                expected,
                observed,
            } => write!(
                formatter,
                "{relation} expected {expected} target, observed {observed}"
            ),
            Self::MissingEstateIdentity => write!(formatter, "authority estate has no identity"),
            Self::AuthorityEstateMismatch => {
                formatter.write_str("authority estate does not match command estate")
            }
            Self::MissingHolderIdentity => write!(formatter, "authority holder has no identity"),
            Self::AuthorityHolderMismatch => {
                formatter.write_str("authority holder does not match command executor")
            }
            Self::AlreadyRecognizedExecutor => write!(formatter, "executor is already recognized"),
            Self::EntityResolution(denial) => denial.fmt(formatter),
            Self::DecisionPlan(denial) => denial.fmt(formatter),
            Self::Traversal(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankExecutorRecognitionProjectionDenial {}
