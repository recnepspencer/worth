use bank_domain::{
    estate::{DeathNoticeId, DeathNoticeStatus, EstateAction, EstateCaseId, EstateCaseStatus},
    schema::{
        BankSchema, DeathNoticeIdentityField, DeathNoticeStatusField, EstateCase,
        EstateCaseIdentityField, EstateCaseStatusField, EstateDeathNotice,
        OpenEstateCaseCapability, OpenEstateCaseOperation,
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

type AdmittedCaseOpening = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    OpenEstateCaseOperation,
    EstateAction,
    EstateCase,
>;
type CaseOpeningEffectProgram = WorthQueryApplicationEffectProgram<
    BankSchema,
    OpenEstateCaseOperation,
    EstateAction,
    EstateCase,
>;

#[derive(Debug)]
pub enum BankEstateCaseOpeningProjectionDenial {
    MissingEstateIdentity,
    EstateMismatch,
    MissingCaseStatus,
    CaseNotPendingOpening,
    NoticeRelationCardinality { expected: usize, observed: usize },
    MissingNoticeIdentity,
    NoticeMismatch,
    MissingNoticeStatus,
    NoticeNotVerified,
    EntityResolution(crate::BankEntityResolutionDenial),
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
}

impl BankIdentityRuntime {
    pub fn open_estate_case(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let command = case_opening_command(action)?;
        let admission = self.admit_case_opening(principal, action, request)?;
        if let Some(outcome) =
            super::idempotency::resolve_admitted_idempotency(self, &admission, idempotency)?
        {
            return Ok(outcome);
        }
        let program = self.materialize_case_opening(admission, command)?;
        Ok(self
            .application_runtime()
            .compare_and_commit_application(program, idempotency)
            .into())
    }

    fn admit_case_opening(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<AdmittedCaseOpening, BankEstateProgressionDenial> {
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                OpenEstateCaseCapability::reference(),
                OpenEstateCaseOperation::reference(),
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
            .installed_operation(OpenEstateCaseOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        self.application_runtime()
            .authorize_capability_operation(
                access,
                &operation,
                TypedMutationPreconditions::<
                    BankSchema,
                    OpenEstateCaseOperation,
                    EstateCase,
                >::default(),
            )
            .map_err(BankEstateProgressionDenial::from_authorization)
    }

    fn materialize_case_opening(
        &self,
        admission: AdmittedCaseOpening,
        command: CaseOpeningCommand,
    ) -> Result<CaseOpeningEffectProgram, BankEstateProgressionDenial> {
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                project_case_opening(reader, estate, command)
            })
            .map_err(BankEstateProgressionDenial::from_projection)?;
        let (projection_result, projection, _) = projected.into_parts();
        projection_result.map_err(BankEstateProgressionDenial::CaseOpeningProjection)?;
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        let estate = reads
            .resolve_entity(EstateCaseIdentityField::reference(), command.estate)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        let mut effects = reads
            .complete_projected_dependencies()
            .map_err(BankEstateProgressionDenial::from_attempt)?
            .begin_effect_program();
        let estate = effects
            .existing_entity(&estate)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        effects
            .write_field(
                &estate,
                EstateCaseStatusField::reference(),
                EstateCaseStatus::Open,
            )
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        effects
            .finish()
            .map_err(BankEstateProgressionDenial::from_attempt)
    }
}

#[derive(Clone, Copy)]
struct CaseOpeningCommand {
    estate: EstateCaseId,
    notice: DeathNoticeId,
}

fn case_opening_command(
    action: EstateAction,
) -> Result<CaseOpeningCommand, BankEstateProgressionDenial> {
    match action {
        EstateAction::OpenEstateCase { estate, notice } => {
            Ok(CaseOpeningCommand { estate, notice })
        }
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "OpenEstateCaseOperation",
        )),
    }
}

fn project_case_opening(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        OpenEstateCaseOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    command: CaseOpeningCommand,
) -> Result<(), BankEstateCaseOpeningProjectionDenial> {
    let observed_estate = reader
        .decision_field(estate, EstateCaseIdentityField::reference())?
        .ok_or(BankEstateCaseOpeningProjectionDenial::MissingEstateIdentity)?;
    if observed_estate != command.estate {
        return Err(BankEstateCaseOpeningProjectionDenial::EstateMismatch);
    }
    let status = reader
        .decision_field(estate, EstateCaseStatusField::reference())?
        .ok_or(BankEstateCaseOpeningProjectionDenial::MissingCaseStatus)?;
    if status != EstateCaseStatus::PendingOpening {
        return Err(BankEstateCaseOpeningProjectionDenial::CaseNotPendingOpening);
    }
    let relations = reader.decision_relations_from(EstateDeathNotice::reference(), estate)?;
    let [relation] = relations.as_slice() else {
        return Err(
            BankEstateCaseOpeningProjectionDenial::NoticeRelationCardinality {
                expected: 1,
                observed: relations.len(),
            },
        );
    };
    let notice = relation.to();
    let observed_notice = reader
        .decision_field(notice, DeathNoticeIdentityField::reference())?
        .ok_or(BankEstateCaseOpeningProjectionDenial::MissingNoticeIdentity)?;
    if observed_notice != command.notice {
        return Err(BankEstateCaseOpeningProjectionDenial::NoticeMismatch);
    }
    let notice_status = reader
        .decision_field(notice, DeathNoticeStatusField::reference())?
        .ok_or(BankEstateCaseOpeningProjectionDenial::MissingNoticeStatus)?;
    if notice_status != DeathNoticeStatus::Verified {
        return Err(BankEstateCaseOpeningProjectionDenial::NoticeNotVerified);
    }
    Ok(())
}

impl From<WorthQueryEntityResolutionDenial> for BankEstateCaseOpeningProjectionDenial {
    fn from(denial: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(crate::BankEntityResolutionDenial::from_query(denial.kind()))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankEstateCaseOpeningProjectionDenial {
    fn from(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            denial.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial> for BankEstateCaseOpeningProjectionDenial {
    fn from(denial: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            denial.kind(),
        ))
    }
}

impl std::fmt::Display for BankEstateCaseOpeningProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEstateIdentity => write!(formatter, "estate case has no identity"),
            Self::EstateMismatch => {
                formatter.write_str("case-opening estate does not match command estate")
            }
            Self::MissingCaseStatus => write!(formatter, "estate case has no status"),
            Self::CaseNotPendingOpening => {
                formatter.write_str("estate case is not pending opening")
            }
            Self::NoticeRelationCardinality { expected, observed } => write!(
                formatter,
                "estate notice relation expected {expected} target, observed {observed}"
            ),
            Self::MissingNoticeIdentity => write!(formatter, "estate notice has no identity"),
            Self::NoticeMismatch => {
                formatter.write_str("estate notice does not match command notice")
            }
            Self::MissingNoticeStatus => write!(formatter, "estate notice has no status"),
            Self::NoticeNotVerified => formatter.write_str("estate notice is not verified"),
            Self::EntityResolution(denial) => denial.fmt(formatter),
            Self::DecisionPlan(denial) => denial.fmt(formatter),
            Self::Traversal(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankEstateCaseOpeningProjectionDenial {}
