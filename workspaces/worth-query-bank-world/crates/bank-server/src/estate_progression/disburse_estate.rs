use bank_domain::{
    estate::{EstateAction, EstateDisbursement, LegalAuthorityId},
    model::BankPrincipalId,
    proposals::{BankIdempotencyClaim, BankIdempotencyKey, BankProposalEngine, BankProposedEffect},
    schema::{
        AccountIdentity, BankSchema, DisburseEstateCapability, DisburseEstateOperation,
        EstateAccount, EstateBeneficiary, EstateCase, EstateCaseIdentityField, EstateExecutor,
        EstateJointOwner, LegalAuthorityEstate, LegalAuthorityHolder, LegalAuthorityIdentityField,
        PrincipalIdentityField,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryApplicationEffectProgram,
        WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationReadAttempt,
        WorthQueryProjectedApplicationMutation,
    },
};

use crate::bank_projection::project_estate_disbursement;
pub use crate::bank_projection::BankEstateDisbursementProjectionDenial;
use crate::operation_commit::{lower_journal, resolve_journal_accounts};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

use super::BankEstateProgressionDenial;

#[cfg(test)]
mod tests;

pub(crate) type AdmittedEstateDisbursement = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    DisburseEstateOperation,
    EstateAction,
    EstateCase,
>;
type EstateDisbursementEffectProgram = WorthQueryApplicationEffectProgram<
    BankSchema,
    DisburseEstateOperation,
    EstateAction,
    EstateCase,
>;

impl BankIdentityRuntime {
    pub fn disburse_estate_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency_key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let idempotency = super::idempotency::disbursement_binding(idempotency_key, action)?;
        self.disburse_estate(principal, action, idempotency, request)
    }

    pub fn disburse_estate(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let admission = self.admit_estate_disbursement(principal, action, request)?;
        if let Some(outcome) =
            super::idempotency::resolve_admitted_idempotency(self, &admission, idempotency)?
        {
            return Ok(outcome);
        }
        let program = self.materialize_estate_disbursement(admission, idempotency)?;
        Ok(self
            .application_runtime()
            .compare_and_commit_application(program, idempotency)
            .into())
    }

    pub(crate) fn admit_estate_disbursement(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<AdmittedEstateDisbursement, BankEstateProgressionDenial> {
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                DisburseEstateCapability::reference(),
                DisburseEstateOperation::reference(),
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
            .installed_operation(DisburseEstateOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        self.application_runtime()
            .authorize_capability_operation(
                access,
                &operation,
                TypedMutationPreconditions::<
                    BankSchema,
                    DisburseEstateOperation,
                    EstateCase,
                >::default(),
            )
            .map_err(BankEstateProgressionDenial::from_authorization)
    }

    pub(crate) fn materialize_estate_disbursement(
        &self,
        admission: AdmittedEstateDisbursement,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<EstateDisbursementEffectProgram, BankEstateProgressionDenial> {
        let command = disbursement_command(*admission.capability_input().ok_or(
            BankEstateProgressionDenial::CommandInput("DisburseEstateOperation admission input"),
        )?)?;
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                project_estate_disbursement(reader, estate, &command)
            })
            .map_err(BankEstateProgressionDenial::from_projection)?;
        let (decision, projection, _) = projected.into_parts();
        let decision =
            decision.map_err(BankEstateProgressionDenial::EstateDisbursementProjection)?;
        let (snapshot, authority, executor) = decision.into_parts();
        let proposal = BankProposalEngine::prepare_estate_disbursement_from_decision(
            snapshot,
            BankIdempotencyClaim::from_application_binding(
                *idempotency.key_identity(),
                *idempotency.intent_identity(),
            ),
            &command,
        )
        .map_err(BankEstateProgressionDenial::Proposal)?;
        let [BankProposedEffect::AppendJournal(journal)] = proposal.effects() else {
            return Err(BankEstateProgressionDenial::CommitPreparation(
                crate::operation_commit::BankCommitPreparationDenial::InvalidProposalShape,
            ));
        };
        let mut reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        observe_disbursement_witnesses(&mut reads, &command, authority, executor)?;
        let accounts = resolve_journal_accounts(&mut reads, journal)
            .map_err(BankEstateProgressionDenial::CommitPreparation)?;
        let mut effects = reads
            .complete_projected_dependencies()
            .map_err(BankEstateProgressionDenial::from_attempt)?
            .begin_effect_program();
        lower_journal(&mut effects, journal, accounts)
            .map_err(BankEstateProgressionDenial::CommitPreparation)?;
        effects
            .finish()
            .map_err(BankEstateProgressionDenial::from_attempt)
    }
}

fn observe_disbursement_witnesses(
    reads: &mut WorthQueryApplicationReadAttempt<
        BankSchema,
        DisburseEstateOperation,
        EstateAction,
        EstateCase,
        WorthQueryProjectedApplicationMutation,
    >,
    command: &EstateDisbursement,
    authority: LegalAuthorityId,
    executor: BankPrincipalId,
) -> Result<(), BankEstateProgressionDenial> {
    let estate = reads.resolve_entity(EstateCaseIdentityField::reference(), command.estate)?;
    let source = reads.resolve_entity(AccountIdentity::reference(), command.source_account)?;
    let destination =
        reads.resolve_entity(AccountIdentity::reference(), command.destination_account)?;
    let beneficiary =
        reads.resolve_entity(PrincipalIdentityField::reference(), command.beneficiary)?;
    let authority = reads.resolve_entity(LegalAuthorityIdentityField::reference(), authority)?;
    let executor = reads.resolve_entity(PrincipalIdentityField::reference(), executor)?;

    require_exact_relation(
        "EstateAccount",
        reads
            .observe_relation(EstateAccount::reference(), &estate, &source)?
            .count(),
    )?;
    require_exact_relation(
        "EstateBeneficiary",
        reads
            .observe_relation(EstateBeneficiary::reference(), &beneficiary, &estate)?
            .count(),
    )?;
    require_exact_relation(
        "EstateJointOwner",
        reads
            .observe_relation(EstateJointOwner::reference(), &beneficiary, &destination)?
            .count(),
    )?;
    require_exact_relation(
        "LegalAuthorityEstate",
        reads
            .observe_relation(LegalAuthorityEstate::reference(), &authority, &estate)?
            .count(),
    )?;
    require_exact_relation(
        "LegalAuthorityHolder",
        reads
            .observe_relation(LegalAuthorityHolder::reference(), &authority, &executor)?
            .count(),
    )?;
    require_exact_relation(
        "EstateExecutor",
        reads
            .observe_relation(EstateExecutor::reference(), &executor, &estate)?
            .count(),
    )?;
    Ok(())
}

fn require_exact_relation(
    relation: &'static str,
    observed: usize,
) -> Result<(), BankEstateProgressionDenial> {
    if observed == 1 {
        return Ok(());
    }
    Err(BankEstateProgressionDenial::EstateDisbursementProjection(
        BankEstateDisbursementProjectionDenial::WitnessRelationCardinality { relation, observed },
    ))
}

fn disbursement_command(
    action: EstateAction,
) -> Result<EstateDisbursement, BankEstateProgressionDenial> {
    match action {
        EstateAction::DisburseEstate(disbursement) => Ok(disbursement),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "DisburseEstateOperation",
        )),
    }
}
