//! Derivation material belongs to the original occurrence's installed mechanism.
pub(super) mod recorded_input;
use super::{
    disclosure::WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedCorrectionSource,
};

use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationCorrectionBinding, ApplicationCorrectionPreImage,
        ApplicationCorrectionSourceMaterial, DeclaredPreImageLocus,
    },
    application_operation::ApplicationMutationBinding,
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    InstalledCorrectionAuthority, InstalledCorrectionMechanism,
};
pub(crate) mod source_outputs;
pub use source_outputs::WorthQueryCorrectionOutputProjectionDenial;

pub(super) fn mechanism<'a, S: ApplicationSchema>(
    source: &'a WorthQuerySelectedCorrectionSource<'_, S>,
) -> Result<&'a InstalledCorrectionMechanism, Denial> {
    let contract = source
        .occurrence()
        .aftermath()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    match contract.authority() {
        InstalledCorrectionAuthority::RuntimeAlone => {}
        InstalledCorrectionAuthority::RuntimeWithExternalOwner
            if contract
                .completed_external_reconciliation()
                .is_some_and(|declared| {
                    contract.reconciliation().is_some_and(|installed| {
                        declared.procedure_slot() == installed.procedure_slot()
                    })
                })
                && contract.correction_disclosure().is_some()
                && matches!(
                    contract.mechanism(),
                    Some(InstalledCorrectionMechanism::Compensation(_))
                )
                && source
                    .occurrence()
                    .memory()
                    .and_then(|value| value.commit_evidence().workflow_origin())
                    .is_some() =>
        {
            match source.application.resolve_correction_external_settlement(source.occurrence().memory().expect("workflow origin belongs to real Memory evidence")) {
                Ok(true) => {},
                Ok(false) => return Err(Denial::Workflow(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
                        "original external workflow operation has no authenticated completion",
                    ),
                )),
                Err(_) => return Err(Denial::UnsupportedSourceMeaning),
            }
        }
        InstalledCorrectionAuthority::RuntimeWithExternalOwner
        | InstalledCorrectionAuthority::NotCorrectable => {
            return Err(Denial::UnsupportedSourceMeaning)
        }
    }
    contract.mechanism().ok_or(Denial::UnsupportedSourceMeaning)
}

pub(super) fn derive<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
    source: &WorthQuerySelectedCorrectionSource<'_, S>,
    request: worth_execution::ExecutionRequest<'_, '_>,
    input: &<C::Source as ApplicationMutationBinding<S>>::Input,
) -> Result<C::InverseIntent, Denial> {
    match mechanism(source)? {
        InstalledCorrectionMechanism::RecordedInverse(inverse)
            if inverse.preimage_demand().loci().is_empty() =>
        {
            let outputs = source_outputs::SourceOutputs::<S, C>::new(source, request);
            let result = C::derive_correction(
                input,
                ApplicationCorrectionSourceMaterial::CreatedEntity(&outputs),
            );
            finish_derivation(result, outputs.take_failure())
        }
        InstalledCorrectionMechanism::RecordedInverse(_) => {
            if !source.occurrence().has_preimage() {
                return Err(Denial::MissingPreImage);
            }
            C::derive_correction(
                input,
                ApplicationCorrectionSourceMaterial::RecordedInverse(&DisclosedPreImage(
                    source.occurrence(),
                )),
            )
            .map_err(Denial::Domain)
        }
        InstalledCorrectionMechanism::Compensation(_) => {
            let outputs = source_outputs::SourceOutputs::<S, C>::new(source, request);
            let result = C::derive_correction(
                input,
                ApplicationCorrectionSourceMaterial::Compensation(&outputs),
            );
            finish_derivation(result, outputs.take_failure())
        }
    }
}

pub fn finish_derivation<T>(
    result: Result<T, worth_query_declaration::facade::application_aftermath::ApplicationCorrectionDerivationDenial>,
    failure: Option<WorthQueryCorrectionOutputProjectionDenial>,
) -> Result<T, Denial> {
    if let Some(cause) = failure {
        return Err(Denial::SourceOutput(cause));
    }
    result.map_err(Denial::Domain)
}

struct DisclosedPreImage<'a>(&'a super::occurrence::CorrectionOccurrence);
impl<S> ApplicationCorrectionPreImage<S> for DisclosedPreImage<'_> {
    fn field(
        &self,
        locus: &DeclaredPreImageLocus<S>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        self.0
            .preimage_value(locus.entity(), locus.aspect(), locus.field())
    }
}
