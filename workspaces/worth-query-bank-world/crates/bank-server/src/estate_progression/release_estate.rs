mod projection;
#[cfg(test)]
mod tests;

pub use projection::BankEstateReleaseProjectionDenial;

use bank_domain::{
    estate::{EstateAction, EstateCaseId, EstateCaseStatus, LegalAuthorityId, MandatoryReviewId},
    model::BankPrincipalId,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, EstateCaseStatusField, EstateExecutor,
        PrincipalIdentityField, ReleaseEstateCapability, ReleaseEstateOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryApplicationEffectProgram,
        WorthQueryApplicationIdempotencyBinding,
    },
};

use self::projection::project_release_readiness;
use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

type AdmittedEstateRelease = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    ReleaseEstateOperation,
    EstateAction,
    EstateCase,
>;
type EstateReleaseEffectProgram = WorthQueryApplicationEffectProgram<
    BankSchema,
    ReleaseEstateOperation,
    EstateAction,
    EstateCase,
>;

impl BankIdentityRuntime {
    pub fn release_estate(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let admission = self.admit_estate_release(principal, action, request)?;
        if let Some(outcome) =
            super::idempotency::resolve_admitted_idempotency(self, &admission, idempotency)?
        {
            return Ok(outcome);
        }
        let program = self.materialize_estate_release(admission)?;
        Ok(self
            .application_runtime()
            .compare_and_commit_application(program, idempotency)
            .into())
    }

    fn admit_estate_release(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<AdmittedEstateRelease, BankEstateProgressionDenial> {
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                ReleaseEstateCapability::reference(),
                ReleaseEstateOperation::reference(),
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
            .installed_operation(ReleaseEstateOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        self.application_runtime()
            .authorize_capability_operation(
                access,
                &operation,
                TypedMutationPreconditions::<
                    BankSchema,
                    ReleaseEstateOperation,
                    EstateCase,
                >::default(),
            )
            .map_err(BankEstateProgressionDenial::from_authorization)
    }

    fn materialize_estate_release(
        &self,
        admission: AdmittedEstateRelease,
    ) -> Result<EstateReleaseEffectProgram, BankEstateProgressionDenial> {
        let command = release_command(*admission.capability_input().ok_or(
            BankEstateProgressionDenial::CommandInput("ReleaseEstateOperation admission input"),
        )?)?;
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, root| {
                project_release_readiness(reader, root, command)
            })
            .map_err(BankEstateProgressionDenial::from_projection)?;
        let (projection_result, projection, _) = projected.into_parts();
        projection_result.map_err(BankEstateProgressionDenial::EstateReleaseProjection)?;
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
        let executor_relation = reads
            .observe_relation(EstateExecutor::reference(), &executor, &estate)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        if executor_relation.count() != 1 {
            return Err(BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::ExecutorRelationCardinality {
                    observed: executor_relation.count(),
                },
            ));
        }
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
                EstateCaseStatus::Released,
            )
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        effects
            .finish()
            .map_err(BankEstateProgressionDenial::from_attempt)
    }
}

#[derive(Clone, Copy)]
pub(super) struct ReleaseCommand {
    estate: EstateCaseId,
    executor: BankPrincipalId,
    authority: LegalAuthorityId,
    review: MandatoryReviewId,
}

fn release_command(action: EstateAction) -> Result<ReleaseCommand, BankEstateProgressionDenial> {
    match action {
        EstateAction::ReleaseEstate {
            estate,
            executor,
            authority,
            review,
        } => Ok(ReleaseCommand {
            estate,
            executor,
            authority,
            review,
        }),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "ReleaseEstateOperation",
        )),
    }
}
