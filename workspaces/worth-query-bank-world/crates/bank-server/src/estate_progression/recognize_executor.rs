use bank_domain::{
    estate::{EstateAction, EstateCaseId, LegalAuthorityId},
    model::BankPrincipalId,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, EstateExecutor, LegalAuthorityEstate,
        LegalAuthorityHolder, LegalAuthorityIdentityField, LegalAuthorityRecognizedField,
        PrincipalIdentityField, RecognizeEstateExecutorCapability,
        RecognizeEstateExecutorOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryApplicationEffectProgram,
        WorthQueryApplicationIdempotencyBinding,
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryEntityResolutionDenial,
        WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
        WorthQueryInvariantProjectionTraversalDenial,
    },
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

type AdmittedRecognitionOperation = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    RecognizeEstateExecutorOperation,
    EstateAction,
    EstateCase,
>;
type RecognitionEffectProgram = WorthQueryApplicationEffectProgram<
    BankSchema,
    RecognizeEstateExecutorOperation,
    EstateAction,
    EstateCase,
>;

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
    pub fn recognize_estate_executor(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let command = recognition_command(action)?;
        let admission = self.admit_recognition_operation(principal, action, request)?;
        if let Some(outcome) =
            super::idempotency::resolve_admitted_idempotency(self, &admission, idempotency)?
        {
            return Ok(outcome);
        }
        let program = self.materialize_recognition_effect(admission, command)?;
        Ok(self
            .application_runtime()
            .compare_and_commit_application(program, idempotency)
            .into())
    }

    fn admit_recognition_operation(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<AdmittedRecognitionOperation, BankEstateProgressionDenial> {
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                RecognizeEstateExecutorCapability::reference(),
                RecognizeEstateExecutorOperation::reference(),
            )
            .map_err(BankEstateProgressionDenial::from_capability_installation)?;
        let selected = self
            .select_current_product()
            .map_err(BankEstateProgressionDenial::from_product_selection)?;
        let access = selected
            .admit_capability_access(principal.query(), &capability, action, request)
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let operation = self
            .application_runtime()
            .installed_schema()
            .installed_operation(RecognizeEstateExecutorOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        self.application_runtime()
            .authorize_capability_operation(
                access,
                &operation,
                TypedMutationPreconditions::<
                    BankSchema,
                    RecognizeEstateExecutorOperation,
                    EstateCase,
                >::default(),
            )
            .map_err(BankEstateProgressionDenial::from_authorization)
    }

    fn materialize_recognition_effect(
        &self,
        admission: AdmittedRecognitionOperation,
        command: RecognitionCommand,
    ) -> Result<RecognitionEffectProgram, BankEstateProgressionDenial> {
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                project_recognition(reader, estate, command)
            })
            .map_err(BankEstateProgressionDenial::from_projection)?;
        let (projection_result, projection, _) = projected.into_parts();
        projection_result.map_err(BankEstateProgressionDenial::ExecutorRecognitionProjection)?;
        let mut reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        let estate = reads
            .resolve_entity(EstateCaseIdentityField::reference(), command.estate)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        let executor = reads
            .resolve_entity(PrincipalIdentityField::reference(), command.executor)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        let existing = reads
            .observe_relation(EstateExecutor::reference(), &executor, &estate)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        if !existing.is_absent() {
            return Err(BankEstateProgressionDenial::ExecutorRecognitionProjection(
                BankExecutorRecognitionProjectionDenial::AlreadyRecognizedExecutor,
            ));
        }
        let mut effects = reads
            .complete_projected_dependencies()
            .map_err(BankEstateProgressionDenial::from_attempt)?
            .begin_effect_program();
        let estate = effects
            .existing_entity(&estate)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        let executor = effects
            .existing_entity(&executor)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        effects
            .link(
                EstateExecutor::reference(),
                format!(
                    "estate-executor:{}:{}",
                    command.estate.get(),
                    command.executor.get()
                ),
                &executor,
                &estate,
            )
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        effects
            .finish()
            .map_err(BankEstateProgressionDenial::from_attempt)
    }
}

#[derive(Clone, Copy)]
struct RecognitionCommand {
    estate: EstateCaseId,
    executor: BankPrincipalId,
    authority: LegalAuthorityId,
}

fn recognition_command(
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

fn project_recognition(
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
    reader.require_decision_relation(EstateExecutor::reference(), &executor, estate)?;
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
