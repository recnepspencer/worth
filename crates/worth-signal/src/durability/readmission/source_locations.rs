//! Exact original membership survives after the startup protected cut is released.
use super::source_catalog::{RecordedAdvanceLocation, RecordedDefinitionLocation};
use super::source_custody::{
    SignalAdvanceLocationReservation, SignalDefinitionLocationReservation,
    SignalFundedAdvanceLocation,
};
use super::{SignalOwnerReadmissionDenial as D, SignalRecordedIdentityContribution};
use crate::durability::publication_binding::SignalCommittedDefinition;
use crate::durability::{
    SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor, SignalDurableMedium,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    SignalBranchAdvanceIdentity, SignalDefinitionTransitionIdentity,
    SignalDefinitionTransitionReference, SignalDurableJournal,
};
use worth_proof::DurablyCommitted;

pub(crate) struct SignalRecoveredSourceLocations {
    pub(crate) frontiers: SignalRecordedIdentityContribution,
    definitions: BTreeMap<SignalDefinitionTransitionIdentity, Arc<SignalDefinitionSourceRecord>>,
    advances: BTreeMap<SignalBranchAdvanceIdentity, SignalFundedAdvanceLocation>,
}
/// An original source membership entry, created only by protected replay or a
/// consumed committed body proof. Sharing this live catalog record is O(1).
pub(crate) struct SignalDefinitionSourceRecord {
    location: RecordedDefinitionLocation,
    _custody: SignalDefinitionLocationReservation,
}
pub(crate) struct SignalAdvanceSourceRecord {
    original: SignalBranchAdvanceIdentity,
    location: RecordedAdvanceLocation,
}
impl SignalDefinitionSourceRecord {
    pub(crate) fn from_committed(
        proof: DurablyCommitted<SignalCommittedDefinition>,
        _custody: SignalDefinitionLocationReservation,
    ) -> Arc<Self> {
        proof.consume(|subject| {
            let (artifact, position) = subject.into_original();
            Arc::new(Self {
                location: RecordedDefinitionLocation::original(
                    position.get(),
                    artifact.reference(),
                    artifact.branch(),
                ),
                _custody,
            })
        })
    }
    pub(crate) fn reference(&self) -> SignalDefinitionTransitionReference {
        self.location.reference
    }
    pub(crate) fn matches(&self, descriptor: &SignalDurableBranchDescriptor) -> bool {
        self.location
            .matches(descriptor.definition(), descriptor.branch())
    }
    pub(crate) fn branch(&self) -> worth_foundational::facade::DurableSignalBranchIdentity {
        self.location.branch()
    }
}
impl SignalRecoveredSourceLocations {
    pub(in crate::durability::readmission) fn from_original(
        definitions: Vec<RecordedDefinitionLocation>,
        advances: BTreeMap<SignalBranchAdvanceIdentity, RecordedAdvanceLocation>,
        frontiers: SignalRecordedIdentityContribution,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, D> {
        let mut recorded_definitions = BTreeMap::new();
        for location in definitions {
            let _custody = SignalDefinitionLocationReservation::reserve(execution)?;
            recorded_definitions.insert(
                location.reference.identity(),
                Arc::new(SignalDefinitionSourceRecord { location, _custody }),
            );
        }
        let mut recorded_advances = BTreeMap::new();
        for (identity, location) in advances {
            let _custody = SignalAdvanceLocationReservation::reserve(execution)?;
            recorded_advances.insert(identity, SignalFundedAdvanceLocation { location, _custody });
        }
        Ok(Self {
            definitions: recorded_definitions,
            advances: recorded_advances,
            frontiers,
        })
    }
    pub(crate) fn record_committed_advance(
        &mut self,
        record: crate::durability::publication_binding::SignalCommittedAdvanceRecord,
        _custody: SignalAdvanceLocationReservation,
    ) -> crate::durability::SignalCommittedAdvanceEvidence {
        let artifact = record.artifact();
        let position = record.position();
        self.advances.insert(
            artifact.identity(),
            SignalFundedAdvanceLocation {
                location: RecordedAdvanceLocation {
                    position: position.get(),
                    digest: artifact.digest(),
                },
                _custody,
            },
        );
        crate::durability::SignalCommittedAdvanceEvidence::from_committed(record)
    }
    pub(crate) fn advance_record(
        &self,
        original: SignalBranchAdvanceIdentity,
    ) -> Result<SignalAdvanceSourceRecord, D> {
        let location = self
            .advances
            .get(&original)
            .ok_or(D::MissingAdvance(original))?
            .location;
        Ok(SignalAdvanceSourceRecord { original, location })
    }
    pub(crate) fn record_committed_definition(
        &mut self,
        proof: DurablyCommitted<SignalCommittedDefinition>,
        _custody: SignalDefinitionLocationReservation,
    ) -> Arc<SignalDefinitionSourceRecord> {
        let record = SignalDefinitionSourceRecord::from_committed(proof, _custody);
        self.definitions
            .insert(record.reference().identity(), Arc::clone(&record));
        record
    }
    pub(crate) fn definition_record(
        &self,
        descriptor: &SignalDurableBranchDescriptor,
    ) -> Result<Arc<SignalDefinitionSourceRecord>, D> {
        self.definitions
            .get(&descriptor.definition().identity())
            .filter(|record| {
                record
                    .location
                    .matches(descriptor.definition(), descriptor.branch())
            })
            .cloned()
            .ok_or(D::MissingDefinition(descriptor.definition()))
    }
}
impl SignalAdvanceSourceRecord {
    pub(crate) fn matches_point(
        &self,
        point: &crate::durability::SignalRecordedSourcePoint,
    ) -> bool {
        self.original == point.reference().identity()
            && self.location.position == point.position()
            && self.location.digest == point.reference().digest()
    }
    pub(crate) fn read(
        &self,
        medium: &dyn SignalDurableMedium,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::durability::SignalBranchAdvanceArtifact, D> {
        if self.original.owner() != medium.binding().owner() {
            return Err(D::ForeignOwner);
        }
        let source = medium
            .capture_recovery_source(execution)
            .map_err(D::Medium)?;
        if source.owner() != medium.binding().owner() {
            return Err(D::ForeignOwner);
        }
        let bytes = source
            .read(
                SignalDurableJournal::Advances,
                self.location.position,
                execution,
            )
            .map_err(D::Medium)?;
        let artifact =
            crate::durability::SignalBranchAdvanceArtifact::readmit_bytes(execution, bytes.bytes())
                .map_err(D::Frame)?;
        if artifact.identity() != self.original || artifact.digest() != self.location.digest {
            return Err(D::MissingAdvance(self.original));
        }
        Ok(artifact)
    }
}
impl SignalDefinitionSourceRecord {
    pub(crate) fn read(
        &self,
        medium: &dyn SignalDurableMedium,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDefinitionTransitionArtifact, D> {
        let reference = self.reference();
        if self.branch().owner() != medium.binding().owner() {
            return Err(D::ForeignOwner);
        }
        let source = medium
            .capture_recovery_source(execution)
            .map_err(D::Medium)?;
        if source.owner() != medium.binding().owner() {
            return Err(D::ForeignOwner);
        }
        let bytes = source
            .read(
                SignalDurableJournal::Definitions,
                self.location.position,
                execution,
            )
            .map_err(D::Medium)?;
        let artifact =
            SignalDefinitionTransitionArtifact::readmit_owned(bytes).map_err(D::Frame)?;
        if artifact.reference() != reference || artifact.branch() != self.branch() {
            return Err(D::MissingDefinition(reference));
        }
        Ok(artifact)
    }
}
