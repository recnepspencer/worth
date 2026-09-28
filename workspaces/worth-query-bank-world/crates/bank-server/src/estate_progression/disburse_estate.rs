pub use crate::bank_projection::BankEstateDisbursementProjectionDenial;
use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{BankSchema, DisburseEstateCapability, DisburseEstateOperation, EstateCase},
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::WorthQueryAdmittedApplicationOperation,
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

pub(crate) type AdmittedEstateDisbursement = WorthQueryAdmittedApplicationOperation<
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
        let EstateAction::DisburseEstate(disbursement) = action else {
            return Err(BankEstateProgressionDenial::CommandInput(
                "DisburseEstateOperation",
            ));
        };
        super::program_outcome::program_outcome(
            self.request(principal, request)
                .mutate(bank_domain::schema::DisburseEstate::new(disbursement))
                .idempotency(idempotency_key)
                .execute_capability_in_program(self.application_program()),
            "DisburseEstateOperation",
            |denial| {
                denial
                    .downcast::<BankEstateDisbursementProjectionDenial>()
                    .map(BankEstateProgressionDenial::EstateDisbursementProjection)
            },
        )
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
}
