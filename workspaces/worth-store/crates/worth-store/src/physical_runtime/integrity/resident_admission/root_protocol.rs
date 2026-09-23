use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_integrity::{
    validate_bootstrap_catalog, BootstrapCatalogIntegrityValidation,
    IntegrityValidatedBootstrapCatalog, PhysicalArtifactScope, UntrustedPhysicalArtifact,
};

use super::{
    denial::ResidentIntegrityAdmissionDenial, load::ResidentAdmissionContext,
    record_binding::ResidentIntegrityRecordBinding,
};

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentBootstrapCatalog<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
    projection: ResidentBootstrapCatalogProjection,
}

#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct ResidentBootstrapCatalogProjection {
    // The admitted scope already binds the record format, so only the root
    // entry is projected.
    pub(in crate::physical_runtime) current_root:
        worth_store_physical_format::CurrentRootCatalogEntry,
}

pub(in crate::physical_runtime) fn admit_resident_bootstrap_catalog<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentBootstrapCatalog<'frame>, ResidentIntegrityAdmissionDenial> {
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_bootstrap_catalog(input, scope).0 {
        BootstrapCatalogIntegrityValidation::Intact(validated) => {
            bind_bootstrap(lease, input, validated, context)
        }
        BootstrapCatalogIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
        }
        BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch) => context.deny(
            ResidentIntegrityAdmissionDenial::BootstrapScopeMismatch(mismatch),
        ),
        BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported) => {
            context.deny(ResidentIntegrityAdmissionDenial::BootstrapUnsupportedFormat(unsupported))
        }
    }
}

fn bind_bootstrap<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedBootstrapCatalog<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentBootstrapCatalog<'frame>, ResidentIntegrityAdmissionDenial> {
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let projection = ResidentBootstrapCatalogProjection {
        current_root: worth_store_physical_format::CurrentRootCatalogEntry::new(
            validated.current_root_generation(),
        ),
    };
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentBootstrapCatalog { source, projection })
}

impl<'frame> IntegrityAdmittedResidentBootstrapCatalog<'frame> {
    pub(in crate::physical_runtime) fn project(
        self,
        context: ResidentAdmissionContext<'_>,
    ) -> Result<ResidentBootstrapCatalogProjection, ResidentIntegrityAdmissionDenial> {
        context.with_owner_projection(self.source, || self.projection)
    }
}
