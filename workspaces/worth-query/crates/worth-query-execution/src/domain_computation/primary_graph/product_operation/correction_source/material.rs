//! Derivation material belongs to the original occurrence's installed mechanism.
use super::{
    disclosure::WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedCorrectionSource,
};
use crate::domain_computation::application_aftermath::retained_preimage::WorthQueryRetainedPreImage;
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

pub(super) fn mechanism<'a, S: ApplicationSchema>(
    source: &'a WorthQuerySelectedCorrectionSource<'_, S>,
) -> Result<&'a InstalledCorrectionMechanism, Denial> {
    let contract = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .installed_aftermath()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    if contract.authority() != InstalledCorrectionAuthority::RuntimeAlone {
        return Err(Denial::UnsupportedSourceMeaning);
    }
    contract.mechanism().ok_or(Denial::UnsupportedSourceMeaning)
}

pub(super) fn derive<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
    source: &WorthQuerySelectedCorrectionSource<'_, S>,
    input: &<C::Source as ApplicationMutationBinding<S>>::Input,
) -> Result<C::InverseIntent, Denial> {
    match mechanism(source)? {
        InstalledCorrectionMechanism::RecordedInverse(_) => {
            let preimage = source
                .occurrence()
                .commit_evidence()
                .retained_preimage()
                .ok_or(Denial::MissingPreImage)?;
            C::derive_correction(
                input,
                ApplicationCorrectionSourceMaterial::RecordedInverse(&DisclosedPreImage(preimage)),
            )
            .map_err(Denial::Domain)
        }
        InstalledCorrectionMechanism::Compensation(_) => {
            C::derive_correction(input, ApplicationCorrectionSourceMaterial::Compensation)
                .map_err(Denial::Domain)
        }
    }
}

struct DisclosedPreImage<'a>(&'a WorthQueryRetainedPreImage);
impl<S> ApplicationCorrectionPreImage<S> for DisclosedPreImage<'_> {
    fn field(
        &self,
        locus: &DeclaredPreImageLocus<S>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        self.0
            .fields()
            .iter()
            .find(|field| {
                field.locus().entity() == locus.entity()
                    && field.locus().aspect() == locus.aspect()
                    && field.locus().field() == locus.field()
            })
            .map(|field| field.value())
    }
}
