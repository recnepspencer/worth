use bank_domain::{
    estate::{DeathNoticeId, DeathNoticeStatus, EstateAction, EstateCaseId, EstateCaseStatus},
    proposals::BankIdempotencyKey,
    schema::{
        BankSchema, DeathNoticeIdentityField, DeathNoticeStatusField, EstateCase,
        EstateCaseIdentityField, EstateCaseStatusField, EstateDeathNotice, OpenEstateCaseOperation,
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
    pub fn open_estate_case_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let EstateAction::OpenEstateCase { estate, notice } = action else {
            return Err(BankEstateProgressionDenial::CommandInput(
                "OpenEstateCaseOperation",
            ));
        };
        super::program_outcome::program_outcome(
            self.request(principal, request)
                .mutate(bank_domain::schema::OpenEstateCase::new(estate, notice))
                .idempotency(key)
                .execute_capability_in_selected_program(self.application_program()),
            "OpenEstateCaseOperation",
            |denial| {
                denial
                    .downcast::<BankEstateCaseOpeningProjectionDenial>()
                    .map(BankEstateProgressionDenial::CaseOpeningProjection)
            },
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CaseOpeningCommand {
    pub(crate) estate: EstateCaseId,
    pub(crate) notice: DeathNoticeId,
}

pub(crate) fn case_opening_command(
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

pub(crate) fn project_case_opening(
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
