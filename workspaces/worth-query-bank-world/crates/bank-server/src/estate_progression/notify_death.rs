use bank_domain::{
    estate::{DeathNoticeId, DeathNoticeStatus, EstateAction, EstateCaseId},
    model::BankPrincipalId,
    proposals::BankIdempotencyKey,
    schema::{
        BankSchema, DeathNoticeIdentityField, DeathNoticeStatusField, DeathNoticeSubject,
        EstateCase, EstateDeathNotice, EstateDeceased, NotifyDeathEstateCapability,
        NotifyDeathEstateOperation, PrincipalIdentityField,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::{
        WorthQueryAdmittedApplicationOperation,
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryEntityResolutionDenial,
        WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
        WorthQueryInvariantProjectionTraversalDenial,
    },
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

type AdmittedNotificationOperation = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    NotifyDeathEstateOperation,
    EstateAction,
    EstateCase,
>;
#[derive(Debug)]
pub enum BankDeathNotificationProjectionDenial {
    RelationCardinality {
        relation: &'static str,
        expected: usize,
        observed: usize,
    },
    MissingNoticeIdentity,
    NoticeMismatch,
    MissingNoticeStatus,
    NoticeNotReported,
    MissingSubjectIdentity(&'static str),
    NoticeSubjectMismatch,
    EstateSubjectMismatch,
    EntityResolution(crate::BankEntityResolutionDenial),
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
}
impl BankIdentityRuntime {
    pub fn notify_estate_death_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let EstateAction::NotifyDeath {
            estate,
            notice,
            subject,
        } = action
        else {
            return Err(BankEstateProgressionDenial::CommandInput(
                "NotifyDeathEstateOperation",
            ));
        };
        super::program_outcome::program_outcome(
            self.request(principal, request)
                .mutate(bank_domain::schema::NotifyEstateDeath::new(
                    estate, notice, subject,
                ))
                .idempotency(key)
                .execute_capability_in_program(self.application_program()),
            "NotifyDeathEstateOperation",
            |denial| {
                denial
                    .downcast::<BankDeathNotificationProjectionDenial>()
                    .map(BankEstateProgressionDenial::DeathNotificationProjection)
            },
        )
    }

    pub(crate) fn admit_notification_operation(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<AdmittedNotificationOperation, BankEstateProgressionDenial> {
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                NotifyDeathEstateCapability::reference(),
                NotifyDeathEstateOperation::reference(),
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
            .installed_operation(NotifyDeathEstateOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        self.application_runtime()
            .authorize_capability_operation(
                access,
                &operation,
                TypedMutationPreconditions::<
                    BankSchema,
                    NotifyDeathEstateOperation,
                    EstateCase,
                >::default(),
            )
            .map_err(BankEstateProgressionDenial::from_authorization)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NotificationCommand {
    pub(crate) estate: EstateCaseId,
    pub(crate) notice: DeathNoticeId,
    pub(crate) subject: BankPrincipalId,
}

pub(crate) fn notification_command(
    action: EstateAction,
) -> Result<NotificationCommand, BankEstateProgressionDenial> {
    match action {
        EstateAction::NotifyDeath {
            estate,
            notice,
            subject,
        } => Ok(NotificationCommand {
            estate,
            notice,
            subject,
        }),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "NotifyDeathEstateOperation",
        )),
    }
}

pub(crate) fn project_notification(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        NotifyDeathEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    command: NotificationCommand,
) -> Result<(), BankDeathNotificationProjectionDenial> {
    let notice = exact_notice(reader, estate, command.notice)?;
    let status = reader
        .decision_field(&notice, DeathNoticeStatusField::reference())?
        .ok_or(BankDeathNotificationProjectionDenial::MissingNoticeStatus)?;
    if status != DeathNoticeStatus::Reported {
        return Err(BankDeathNotificationProjectionDenial::NoticeNotReported);
    }
    require_notice_subject(reader, &notice, command.subject)?;
    require_estate_subject(reader, estate, command.subject)
}

fn exact_notice(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        NotifyDeathEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    expected: DeathNoticeId,
) -> Result<
    WorthQueryInvariantEntityIdentity<BankSchema, bank_domain::schema::DeathNotice>,
    BankDeathNotificationProjectionDenial,
> {
    let relations = reader.decision_relations_from(EstateDeathNotice::reference(), estate)?;
    let [relation] = relations.as_slice() else {
        return Err(relation_cardinality("EstateDeathNotice", relations.len()));
    };
    let notice = relation.to().clone();
    let observed = reader
        .decision_field(&notice, DeathNoticeIdentityField::reference())?
        .ok_or(BankDeathNotificationProjectionDenial::MissingNoticeIdentity)?;
    if observed != expected {
        return Err(BankDeathNotificationProjectionDenial::NoticeMismatch);
    }
    Ok(notice)
}

fn require_notice_subject(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        NotifyDeathEstateOperation,
    >,
    notice: &WorthQueryInvariantEntityIdentity<BankSchema, bank_domain::schema::DeathNotice>,
    expected: BankPrincipalId,
) -> Result<(), BankDeathNotificationProjectionDenial> {
    let relations = reader.decision_relations_from(DeathNoticeSubject::reference(), notice)?;
    let [relation] = relations.as_slice() else {
        return Err(relation_cardinality("DeathNoticeSubject", relations.len()));
    };
    let observed = reader
        .decision_field(relation.to(), PrincipalIdentityField::reference())?
        .ok_or(BankDeathNotificationProjectionDenial::MissingSubjectIdentity("death notice"))?;
    if observed != expected {
        return Err(BankDeathNotificationProjectionDenial::NoticeSubjectMismatch);
    }
    Ok(())
}

fn require_estate_subject(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        NotifyDeathEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    expected: BankPrincipalId,
) -> Result<(), BankDeathNotificationProjectionDenial> {
    let relations = reader.decision_relations_from(EstateDeceased::reference(), estate)?;
    let [relation] = relations.as_slice() else {
        return Err(relation_cardinality("EstateDeceased", relations.len()));
    };
    let observed = reader
        .decision_field(relation.to(), PrincipalIdentityField::reference())?
        .ok_or(BankDeathNotificationProjectionDenial::MissingSubjectIdentity("estate"))?;
    if observed != expected {
        return Err(BankDeathNotificationProjectionDenial::EstateSubjectMismatch);
    }
    Ok(())
}

fn relation_cardinality(
    relation: &'static str,
    observed: usize,
) -> BankDeathNotificationProjectionDenial {
    BankDeathNotificationProjectionDenial::RelationCardinality {
        relation,
        expected: 1,
        observed,
    }
}

impl From<WorthQueryEntityResolutionDenial> for BankDeathNotificationProjectionDenial {
    fn from(denial: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(crate::BankEntityResolutionDenial::from_query(denial.kind()))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankDeathNotificationProjectionDenial {
    fn from(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            denial.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial> for BankDeathNotificationProjectionDenial {
    fn from(denial: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            denial.kind(),
        ))
    }
}

impl std::fmt::Display for BankDeathNotificationProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RelationCardinality {
                relation,
                expected,
                observed,
            } => write!(
                formatter,
                "{relation} expected {expected} target, observed {observed}"
            ),
            Self::MissingNoticeIdentity => write!(formatter, "death notice has no identity"),
            Self::NoticeMismatch => {
                formatter.write_str("estate notice does not match command notice")
            }
            Self::MissingNoticeStatus => write!(formatter, "death notice has no status"),
            Self::NoticeNotReported => formatter.write_str("death notice is not reportable"),
            Self::MissingSubjectIdentity(owner) => {
                write!(formatter, "{owner} subject has no identity")
            }
            Self::NoticeSubjectMismatch => {
                formatter.write_str("notice subject does not match command subject")
            }
            Self::EstateSubjectMismatch => {
                formatter.write_str("estate subject does not match command subject")
            }
            Self::EntityResolution(denial) => denial.fmt(formatter),
            Self::DecisionPlan(denial) => denial.fmt(formatter),
            Self::Traversal(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankDeathNotificationProjectionDenial {}
