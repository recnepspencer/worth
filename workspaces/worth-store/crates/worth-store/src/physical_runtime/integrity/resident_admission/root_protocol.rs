use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_integrity::{
    validate_bootstrap_catalog, validate_current_root_selector, validate_previous_root_selector,
    BootstrapCatalogIntegrityValidation, CurrentRootSelectorIntegrityValidation,
    IntegrityValidatedBootstrapCatalog, IntegrityValidatedCurrentRootSelector,
    IntegrityValidatedPreviousRootSelector, PhysicalArtifactScope,
    PreviousRootSelectorIntegrityValidation, UntrustedPhysicalArtifact,
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
    pub(in crate::physical_runtime) record_format:
        worth_store_physical_format::PhysicalRecordFormatDeclaration,
    pub(in crate::physical_runtime) current_root:
        worth_store_physical_format::CurrentRootCatalogEntry,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentCurrentRootSelector<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentPreviousRootSelector<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentCurrentRootSelectorView<'frame> {
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentPreviousRootSelectorView<'frame> {
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
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

pub(in crate::physical_runtime) fn admit_resident_current_root_selector<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentCurrentRootSelector<'frame>, ResidentIntegrityAdmissionDenial>
{
    if let Some(source) = context.reuse(lease, scope)? {
        return Ok(IntegrityAdmittedResidentCurrentRootSelector { source });
    }
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_current_root_selector(input, scope).0 {
        CurrentRootSelectorIntegrityValidation::Intact(validated) => {
            bind_current_selector(lease, input, validated, context)
        }
        CurrentRootSelectorIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
        }
    }
}

pub(in crate::physical_runtime) fn admit_resident_previous_root_selector<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentPreviousRootSelector<'frame>, ResidentIntegrityAdmissionDenial>
{
    if let Some(source) = context.reuse(lease, scope)? {
        return Ok(IntegrityAdmittedResidentPreviousRootSelector { source });
    }
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_previous_root_selector(input, scope).0 {
        PreviousRootSelectorIntegrityValidation::Intact(validated) => {
            bind_previous_selector(lease, input, validated, context)
        }
        PreviousRootSelectorIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
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
        record_format: validated.record_format(),
        current_root: worth_store_physical_format::CurrentRootCatalogEntry::new(
            validated.current_root_generation(),
        ),
    };
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentBootstrapCatalog { source, projection })
}

fn bind_current_selector<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedCurrentRootSelector<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentCurrentRootSelector<'frame>, ResidentIntegrityAdmissionDenial>
{
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentCurrentRootSelector { source })
}

fn bind_previous_selector<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedPreviousRootSelector<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentPreviousRootSelector<'frame>, ResidentIntegrityAdmissionDenial>
{
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentPreviousRootSelector { source })
}

impl<'frame> IntegrityAdmittedResidentBootstrapCatalog<'frame> {
    pub(in crate::physical_runtime) fn project(
        self,
        context: ResidentAdmissionContext<'_>,
    ) -> Result<ResidentBootstrapCatalogProjection, ResidentIntegrityAdmissionDenial> {
        context.with_owner_projection(self.source, || self.projection)
    }
}

impl<'frame> IntegrityAdmittedResidentCurrentRootSelector<'frame> {
    pub(in crate::physical_runtime) fn with_owner_decoder<T>(
        self,
        context: ResidentAdmissionContext<'_>,
        decoder: impl for<'view> FnOnce(IntegrityAdmittedResidentCurrentRootSelectorView<'view>) -> T,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        context.with_owner_decoder(self.source, |lease, scope| {
            decoder(IntegrityAdmittedResidentCurrentRootSelectorView { lease, scope })
        })
    }
}

impl<'frame> IntegrityAdmittedResidentPreviousRootSelector<'frame> {
    pub(in crate::physical_runtime) fn with_owner_decoder<T>(
        self,
        context: ResidentAdmissionContext<'_>,
        decoder: impl for<'view> FnOnce(IntegrityAdmittedResidentPreviousRootSelectorView<'view>) -> T,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        context.with_owner_decoder(self.source, |lease, scope| {
            decoder(IntegrityAdmittedResidentPreviousRootSelectorView { lease, scope })
        })
    }
}

macro_rules! resident_root_protocol_view {
    ($view:ty) => {
        impl $view {
            pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
                self.lease
            }

            pub(in crate::physical_runtime) const fn scope(&self) -> PhysicalArtifactScope {
                self.scope
            }
        }
    };
}

resident_root_protocol_view!(IntegrityAdmittedResidentCurrentRootSelectorView<'_>);
resident_root_protocol_view!(IntegrityAdmittedResidentPreviousRootSelectorView<'_>);
