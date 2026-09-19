use bank_domain::{
    estate::{DeathNoticeId, DeathNoticeStatus, EstateAction},
    model::BankPrincipalId,
    proposals::BankIdempotencyKey,
    schema::{
        BankSchema, DeathNoticeIdentityField, DeathNoticeStatusField, DeathNoticeSubject,
        EstateCase, EstateDeathNotice, EstateDeceased, PrincipalIdentityField,
        RetransmitDeathNoticeEstateOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    primary_graph::{
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    },
};

use super::{notify_death::BankDeathNotificationProjectionDenial, BankEstateProgressionDenial};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

impl BankIdentityRuntime {
    pub fn retransmit_estate_death_notice_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let EstateAction::RetransmitDeathNotice {
            estate,
            notice,
            subject,
        } = action
        else {
            return Err(BankEstateProgressionDenial::CommandInput(
                "RetransmitDeathNoticeEstateOperation",
            ));
        };
        super::program_outcome::program_outcome(
            self.request(principal, request)
                .mutate(bank_domain::schema::RetransmitEstateDeathNotice::new(
                    estate, notice, subject,
                ))
                .idempotency(key)
                .execute_capability_in_program(self.application_program()),
            "RetransmitDeathNoticeEstateOperation",
            |denial| {
                denial
                    .downcast::<BankDeathNotificationProjectionDenial>()
                    .map(BankEstateProgressionDenial::DeathNotificationProjection)
            },
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RetransmitCommand {
    pub(crate) notice: DeathNoticeId,
    pub(crate) subject: BankPrincipalId,
}

pub(crate) fn retransmit_command(
    action: EstateAction,
) -> Result<RetransmitCommand, BankEstateProgressionDenial> {
    match action {
        EstateAction::RetransmitDeathNotice {
            estate: _,
            notice,
            subject,
        } => Ok(RetransmitCommand { notice, subject }),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "RetransmitDeathNoticeEstateOperation",
        )),
    }
}

pub(crate) fn project_retransmit(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        RetransmitDeathNoticeEstateOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    command: RetransmitCommand,
) -> Result<(), BankDeathNotificationProjectionDenial> {
    let notice = exact_notice(reader, estate, command.notice)?;
    let status = reader
        .decision_field(&notice, DeathNoticeStatusField::reference())?
        .ok_or(BankDeathNotificationProjectionDenial::MissingNoticeStatus)?;
    if status != DeathNoticeStatus::NotificationRequested {
        return Err(BankDeathNotificationProjectionDenial::NoticeNotReported);
    }
    require_notice_subject(reader, &notice, command.subject)?;
    require_estate_subject(reader, estate, command.subject)
}

fn exact_notice(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        RetransmitDeathNoticeEstateOperation,
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
        RetransmitDeathNoticeEstateOperation,
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
        RetransmitDeathNoticeEstateOperation,
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
