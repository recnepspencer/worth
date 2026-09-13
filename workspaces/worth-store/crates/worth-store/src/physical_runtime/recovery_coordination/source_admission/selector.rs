use worth_store_physical_backend::CompletedScheduledRecoveryReopenRead;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurableRootSelector, PhysicalRecordFormatDeclaration,
    RecordArtifactFile, RootSelectorIdentity, RootSelectorRole,
};
use worth_store_physical_integrity::{
    validate_current_root_selector, validate_previous_root_selector,
    CurrentRootSelectorIntegrityValidation, IntegrityValidatedCurrentRootSelector,
    IntegrityValidatedPreviousRootSelector, PhysicalArtifactScope, PhysicalByteRange,
    PreviousRootSelectorIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::completed_read::{BoundScheduledRootProtocolSource, ScheduledRootProtocolSource};
use crate::physical_runtime::RootProtocolAdmissionDenial;

pub(in crate::physical_runtime::recovery_coordination) struct IntegrityAdmittedCurrentRootSelector<
    'source,
> {
    source: ScheduledRootProtocolSource<'source>,
    projection: AdmittedSelectorProjection,
}

pub(in crate::physical_runtime::recovery_coordination) struct IntegrityAdmittedPreviousRootSelector<
    'source,
> {
    source: ScheduledRootProtocolSource<'source>,
    projection: AdmittedSelectorProjection,
}

#[derive(Clone, Copy)]
struct AdmittedSelectorProjection {
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    identity: RootSelectorIdentity,
    role: RootSelectorRole,
    root_generation: u64,
    linked_selector: Option<RootSelectorIdentity>,
    linked_root_generation: Option<u64>,
}

pub(in crate::physical_runtime::recovery_coordination) fn admit_scheduled_current_selector(
    read: &CompletedScheduledRecoveryReopenRead,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<IntegrityAdmittedCurrentRootSelector<'_>, RootProtocolAdmissionDenial> {
    let (input, scope) =
        selector_input(read, store, format, RecordArtifactFile::CurrentRootSelector)?;
    let source = BoundScheduledRootProtocolSource::bind(
        read,
        RecordArtifactFile::CurrentRootSelector,
        scope,
    )?;
    let (validation, _) = validate_current_root_selector(input, scope);
    let validated = match validation {
        CurrentRootSelectorIntegrityValidation::Intact(validated) => validated,
        CurrentRootSelectorIntegrityValidation::Rejected(rejection) => {
            return Err(RootProtocolAdmissionDenial::from_validation(rejection))
        }
    };
    if !validated.matches_input(input) {
        return Err(RootProtocolAdmissionDenial::SourceIncarnationMismatch);
    }
    let projection = current_projection(&validated);
    let source = source.admit(validated.into_validation_record())?;
    Ok(IntegrityAdmittedCurrentRootSelector { source, projection })
}

pub(in crate::physical_runtime::recovery_coordination) fn admit_scheduled_previous_selector(
    read: &CompletedScheduledRecoveryReopenRead,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<IntegrityAdmittedPreviousRootSelector<'_>, RootProtocolAdmissionDenial> {
    let (input, scope) = selector_input(
        read,
        store,
        format,
        RecordArtifactFile::PreviousRootSelector,
    )?;
    let source = BoundScheduledRootProtocolSource::bind(
        read,
        RecordArtifactFile::PreviousRootSelector,
        scope,
    )?;
    let (validation, _) = validate_previous_root_selector(input, scope);
    let validated = match validation {
        PreviousRootSelectorIntegrityValidation::Intact(validated) => validated,
        PreviousRootSelectorIntegrityValidation::Rejected(rejection) => {
            return Err(RootProtocolAdmissionDenial::from_validation(rejection))
        }
    };
    if !validated.matches_input(input) {
        return Err(RootProtocolAdmissionDenial::SourceIncarnationMismatch);
    }
    let projection = previous_projection(&validated);
    let source = source.admit(validated.into_validation_record())?;
    Ok(IntegrityAdmittedPreviousRootSelector { source, projection })
}

fn selector_input(
    read: &CompletedScheduledRecoveryReopenRead,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    artifact: RecordArtifactFile,
) -> Result<(UntrustedPhysicalArtifact<'_>, PhysicalArtifactScope), RootProtocolAdmissionDenial> {
    if read.artifact() != artifact {
        return Err(RootProtocolAdmissionDenial::SourceArtifactMismatch);
    }
    let range = PhysicalByteRange::new(0, read.bytes().len() as u64)
        .map_err(|_| RootProtocolAdmissionDenial::SourceRangeMismatch)?;
    let scope = match artifact {
        RecordArtifactFile::CurrentRootSelector => {
            PhysicalArtifactScope::current_root_selector(store, format, range)
        }
        RecordArtifactFile::PreviousRootSelector => {
            PhysicalArtifactScope::previous_root_selector(store, format, range)
        }
        _ => return Err(RootProtocolAdmissionDenial::SourceArtifactMismatch),
    };
    Ok((
        UntrustedPhysicalArtifact::from_bounded_bytes(read.bytes()),
        scope,
    ))
}

fn current_projection(
    validated: &IntegrityValidatedCurrentRootSelector<'_>,
) -> AdmittedSelectorProjection {
    AdmittedSelectorProjection {
        store: validated.scope().store_identity(),
        format: validated.record_format(),
        identity: validated.selector_identity(),
        role: RootSelectorRole::Current,
        root_generation: validated.root_generation(),
        linked_selector: validated.linked_selector(),
        linked_root_generation: validated.linked_root_generation(),
    }
}

fn previous_projection(
    validated: &IntegrityValidatedPreviousRootSelector<'_>,
) -> AdmittedSelectorProjection {
    AdmittedSelectorProjection {
        store: validated.scope().store_identity(),
        format: validated.record_format(),
        identity: validated.selector_identity(),
        role: RootSelectorRole::Previous,
        root_generation: validated.root_generation(),
        linked_selector: validated.linked_selector(),
        linked_root_generation: validated.linked_root_generation(),
    }
}

impl IntegrityAdmittedCurrentRootSelector<'_> {
    pub(in crate::physical_runtime::recovery_coordination) fn project(
        self,
    ) -> Result<DurableRootSelector, RootProtocolAdmissionDenial> {
        let _source_incarnation = (self.source.operation(), self.source.validation());
        self.projection.project()
    }
}

impl IntegrityAdmittedPreviousRootSelector<'_> {
    pub(in crate::physical_runtime::recovery_coordination) fn project(
        self,
    ) -> Result<DurableRootSelector, RootProtocolAdmissionDenial> {
        let _source_incarnation = (self.source.operation(), self.source.validation());
        self.projection.project()
    }
}

impl AdmittedSelectorProjection {
    fn project(self) -> Result<DurableRootSelector, RootProtocolAdmissionDenial> {
        DurableRootSelector::new(
            self.store,
            self.format,
            self.identity,
            self.role,
            self.root_generation,
            self.linked_selector,
            self.linked_root_generation,
        )
        .ok_or(RootProtocolAdmissionDenial::OwnerProjectionRejected)
    }
}
