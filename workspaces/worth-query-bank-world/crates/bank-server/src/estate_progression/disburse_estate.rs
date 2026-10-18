pub use crate::bank_projection::BankEstateDisbursementProjectionDenial;
use bank_domain::{
    estate::EstateAction,
    schema::{BankSchema, DisburseEstateCapability, DisburseEstateOperation, EstateCase},
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::WorthQueryAdmittedApplicationOperation,
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

pub(crate) type AdmittedEstateDisbursement = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    DisburseEstateOperation,
    EstateAction,
    EstateCase,
>;

impl BankIdentityRuntime {
    pub(crate) fn admit_estate_disbursement(
        &self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
            .select_current_product(phase)
            .map_err(BankEstateProgressionDenial::from_product_selection)?;
        let access = selected
            .admit_capability_access(phase, principal.query(), &capability, action, request)
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let operation = self
            .application_runtime()
            .installed_schema()
            .installed_operation(DisburseEstateOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        self.application_runtime()
            .authorize_capability_operation(phase,
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
