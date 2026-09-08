use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_integrity::{
    validate_free_space_header, validate_free_space_membership_block,
    FreeSpaceHeaderIntegrityValidation, FreeSpaceMembershipBlockIntegrityValidation,
    IntegrityValidatedFreeSpaceHeader, IntegrityValidatedFreeSpaceMembershipBlock,
    PhysicalArtifactScope, UntrustedPhysicalArtifact,
};

use super::{
    denial::ResidentIntegrityAdmissionDenial, load::ResidentAdmissionContext,
    record_binding::ResidentIntegrityRecordBinding,
};

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentFreeSpaceHeader<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentFreeSpaceMembershipBlock<'frame> {
    source: ResidentIntegrityRecordBinding<'frame>,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentFreeSpaceHeaderView<'frame> {
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
}

pub(in crate::physical_runtime) struct IntegrityAdmittedResidentFreeSpaceMembershipView<'frame> {
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
}

pub(in crate::physical_runtime) fn admit_resident_free_space_header<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentFreeSpaceHeader<'frame>, ResidentIntegrityAdmissionDenial> {
    if let Some(source) = context.reuse(lease, scope)? {
        return Ok(IntegrityAdmittedResidentFreeSpaceHeader { source });
    }
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_free_space_header(input, scope).0 {
        FreeSpaceHeaderIntegrityValidation::Intact(validated) => {
            bind_free_space_header(lease, input, validated, context)
        }
        FreeSpaceHeaderIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
        }
    }
}

pub(in crate::physical_runtime) fn admit_resident_free_space_membership_block<'frame>(
    lease: &'frame PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    context: ResidentAdmissionContext<'_>,
) -> Result<
    IntegrityAdmittedResidentFreeSpaceMembershipBlock<'frame>,
    ResidentIntegrityAdmissionDenial,
> {
    if let Some(source) = context.reuse(lease, scope)? {
        return Ok(IntegrityAdmittedResidentFreeSpaceMembershipBlock { source });
    }
    let input = context.exact_input(lease, scope)?;
    context.observe_fresh_validation();
    match validate_free_space_membership_block(input, scope).0 {
        FreeSpaceMembershipBlockIntegrityValidation::Intact(validated) => {
            bind_free_space_membership(lease, input, validated, context)
        }
        FreeSpaceMembershipBlockIntegrityValidation::Rejected(rejection) => {
            context.validation_rejected(rejection)
        }
    }
}

fn bind_free_space_header<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedFreeSpaceHeader<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<IntegrityAdmittedResidentFreeSpaceHeader<'frame>, ResidentIntegrityAdmissionDenial> {
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentFreeSpaceHeader { source })
}

fn bind_free_space_membership<'frame>(
    lease: &'frame PhysicalFrameLease,
    input: UntrustedPhysicalArtifact<'frame>,
    validated: IntegrityValidatedFreeSpaceMembershipBlock<'frame>,
    context: ResidentAdmissionContext<'_>,
) -> Result<
    IntegrityAdmittedResidentFreeSpaceMembershipBlock<'frame>,
    ResidentIntegrityAdmissionDenial,
> {
    if !validated.matches_input(input) {
        return context.deny(ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch);
    }
    let scope = validated.scope();
    let source = context.bind_validated(lease, scope, validated.into_validation_record())?;
    Ok(IntegrityAdmittedResidentFreeSpaceMembershipBlock { source })
}

impl<'frame> IntegrityAdmittedResidentFreeSpaceHeader<'frame> {
    pub(in crate::physical_runtime) fn with_owner_decoder<T>(
        self,
        context: ResidentAdmissionContext<'_>,
        decoder: impl for<'view> FnOnce(IntegrityAdmittedResidentFreeSpaceHeaderView<'view>) -> T,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        context.with_owner_decoder(self.source, |lease, scope| {
            decoder(IntegrityAdmittedResidentFreeSpaceHeaderView { lease, scope })
        })
    }
}

impl<'frame> IntegrityAdmittedResidentFreeSpaceMembershipBlock<'frame> {
    pub(in crate::physical_runtime) fn with_owner_decoder<T>(
        self,
        context: ResidentAdmissionContext<'_>,
        decoder: impl for<'view> FnOnce(IntegrityAdmittedResidentFreeSpaceMembershipView<'view>) -> T,
    ) -> Result<T, ResidentIntegrityAdmissionDenial> {
        context.with_owner_decoder(self.source, |lease, scope| {
            decoder(IntegrityAdmittedResidentFreeSpaceMembershipView { lease, scope })
        })
    }
}

impl IntegrityAdmittedResidentFreeSpaceHeaderView<'_> {
    pub(in crate::physical_runtime) fn project_header(
        &self,
        capacity: u16,
    ) -> Result<
        (
            worth_store_physical_format::DurableFreeSpaceManifestHeader,
            worth_store_physical_format::PhysicalRecordFormatDeclaration,
        ),
        worth_store_physical_format::FreeSpaceRoutingDenial,
    > {
        use worth_store_physical_format::{
            DurableFreeSpaceManifestHeader, DURABLE_FRAME_HEADER_BYTES,
        };
        let identity = self
            .scope
            .free_space_header_identity()
            .expect("admitted header scope");
        let format = self
            .scope
            .durable_frame_record_format()
            .expect("admitted durable format");
        DurableFreeSpaceManifestHeader::project_payload(
            &self.lease[DURABLE_FRAME_HEADER_BYTES..],
            identity.generation().get(),
            format,
            capacity,
        )
        .map(|header| (header, format))
    }

    pub(in crate::physical_runtime) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
}

impl IntegrityAdmittedResidentFreeSpaceMembershipView<'_> {
    pub(in crate::physical_runtime) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
}

impl IntegrityAdmittedResidentFreeSpaceMembershipView<'_> {
    pub(in crate::physical_runtime) fn project_block(
        &self,
        capacity: u16,
    ) -> Result<
        worth_store_physical_format::PhysicalFreeSpaceMembershipBlock,
        worth_store_physical_format::BoundedFreeSpaceMembershipBlockDecodeDenial,
    > {
        use worth_store_physical_format::{
            FreeSpaceMembershipBlockDecodeLimits, PhysicalFreeSpaceMembershipBlock,
            DURABLE_FRAME_HEADER_BYTES,
        };
        let identity = self
            .scope
            .free_space_membership_block_identity()
            .expect("admitted family scope");
        PhysicalFreeSpaceMembershipBlock::project_payload(
            &self.lease[DURABLE_FRAME_HEADER_BYTES..],
            identity.reference().block(),
            capacity,
            FreeSpaceMembershipBlockDecodeLimits {
                leaf_entries: u64::from(capacity),
                branch_children: u64::from(capacity),
            },
        )
    }
}
