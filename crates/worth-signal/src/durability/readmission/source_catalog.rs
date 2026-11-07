//! Replay original journals in physical order; retain coordinates, not result storage.
use super::{SignalOwnerReadmissionDenial as D, SignalRecordedIdentityContribution};
use crate::durability::{
    SignalAuthoredBranchChange, SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact,
    SignalDurableBranchDescriptor, SignalDurableMedium, SignalDurableReadSource,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalSnapshotIdentity, SignalDefinitionTransitionIdentity,
    SignalDefinitionTransitionReference, SignalDurableJournal,
};

#[derive(Clone, Debug)]
pub(crate) struct RecordedDefinitionLocation {
    pub(crate) position: u64,
    pub(crate) reference: SignalDefinitionTransitionReference,
    branch: DurableSignalBranchIdentity,
}
impl RecordedDefinitionLocation {
    pub(in crate::durability::readmission) fn original(
        position: u64,
        reference: SignalDefinitionTransitionReference,
        branch: DurableSignalBranchIdentity,
    ) -> Self {
        Self {
            position,
            reference,
            branch,
        }
    }
    pub(in crate::durability::readmission) fn matches(
        &self,
        reference: SignalDefinitionTransitionReference,
        branch: DurableSignalBranchIdentity,
    ) -> bool {
        self.reference == reference && self.branch == branch
    }
    pub(in crate::durability::readmission) fn branch(&self) -> DurableSignalBranchIdentity {
        self.branch
    }
}
#[derive(Clone, Copy)]
pub(in crate::durability::readmission) struct RecordedAdvanceLocation {
    pub(in crate::durability::readmission) position: u64,
    pub(in crate::durability::readmission) digest: [u8; 32],
}
#[derive(Clone, Debug)]
pub(crate) struct RecoveredAuthoredBranch {
    pub(crate) name: String,
    pub(crate) parent: Option<SignalDurableBranchDescriptor>,
    pub(crate) head: SignalDurableBranchDescriptor,
    pub(crate) restored: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct RecoveredAuthoredSnapshot {
    pub(crate) descriptor: SignalDurableBranchDescriptor,
    pub(crate) name: String,
    pub(crate) parent: Option<SignalDurableBranchDescriptor>,
}
pub(crate) struct SignalOwnerSourceReadmission {
    original: Box<dyn SignalDurableReadSource>,
    pub(crate) definitions: Vec<RecordedDefinitionLocation>,
    definitions_by_identity: BTreeMap<SignalDefinitionTransitionIdentity, usize>,
    advances:
        BTreeMap<worth_foundational::facade::SignalBranchAdvanceIdentity, RecordedAdvanceLocation>,
    pub(crate) branches: BTreeMap<DurableSignalBranchIdentity, RecoveredAuthoredBranch>,
    pub(crate) snapshots: BTreeMap<DurableSignalSnapshotIdentity, RecoveredAuthoredSnapshot>,
    pub(crate) frontiers: SignalRecordedIdentityContribution,
    _storage: Option<worth_execution::ExecutionMemoryReservation>,
}
impl SignalOwnerSourceReadmission {
    pub(crate) fn into_locations(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<super::SignalRecoveredSourceLocations, D> {
        let Self {
            original,
            definitions,
            advances,
            frontiers,
            _storage,
            branches,
            snapshots,
            definitions_by_identity,
        } = self;
        let locations = super::SignalRecoveredSourceLocations::from_original(
            definitions,
            advances,
            frontiers,
            execution,
        )?;
        drop((original, branches, snapshots, definitions_by_identity));
        drop(_storage);
        Ok(locations)
    }
    pub(crate) fn capture(
        medium: &dyn SignalDurableMedium,
        contribution: SignalRecordedIdentityContribution,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, D> {
        match medium.publication_task_posture() {
            crate::durability::SignalPublicationTaskPosture::Terminal => {}
            posture @ (crate::durability::SignalPublicationTaskPosture::Busy
            | crate::durability::SignalPublicationTaskPosture::Unsettled) => {
                return Err(D::PublicationNotQuiescent(posture))
            }
        }
        let original = medium
            .capture_recovery_source(execution)
            .map_err(D::Medium)?;
        if original.owner() != medium.binding().owner() || contribution.owner() != original.owner()
        {
            return Err(D::ForeignOwner);
        }
        use crate::data::retained_storage::{
            btree_structure_charge, RetainedStorageCharge as Charge,
        };
        let definitions = usize::try_from(
            original
                .frontier(SignalDurableJournal::Definitions, execution)
                .map_err(D::Medium)?,
        )
        .map_err(|_| D::MetadataCapacity)?;
        let advances = usize::try_from(
            original
                .frontier(SignalDurableJournal::Advances, execution)
                .map_err(D::Medium)?,
        )
        .map_err(|_| D::MetadataCapacity)?;
        let storage = Charge::capacity::<RecordedDefinitionLocation>(definitions)
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<
                    SignalDefinitionTransitionIdentity,
                    usize,
                >(definitions)?)
            })
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<
                    worth_foundational::facade::SignalBranchAdvanceIdentity,
                    RecordedAdvanceLocation,
                >(advances)?)
            })
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<
                    DurableSignalBranchIdentity,
                    RecoveredAuthoredBranch,
                >(advances)?)
            })
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<
                    DurableSignalSnapshotIdentity,
                    RecoveredAuthoredSnapshot,
                >(advances)?)
            })
            .map_err(|_| D::MetadataCapacity)?;
        let _storage = execution
            .reserve_retained_memory(storage.bytes())
            .map_err(D::Request)?;
        let mut recovered = Self {
            original,
            definitions: Vec::with_capacity(definitions),
            definitions_by_identity: BTreeMap::new(),
            advances: BTreeMap::new(),
            branches: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            frontiers: contribution,
            _storage,
        };
        recovered.read_definitions(execution)?;
        recovered.read_advances(execution)?;
        recovered.frontiers.check_issuable()?;
        Ok(recovered)
    }
    fn read_definitions(&mut self, execution: &ExecutionRequest<'_, '_>) -> Result<(), D> {
        let end = self
            .original
            .frontier(SignalDurableJournal::Definitions, execution)
            .map_err(D::Medium)?;
        for position in 1..=end {
            let bytes = self
                .original
                .read(SignalDurableJournal::Definitions, position, execution)
                .map_err(D::Medium)?;
            let artifact =
                SignalDefinitionTransitionArtifact::readmit_owned(bytes).map_err(D::Frame)?;
            self.frontiers.observe_transition(&artifact)?;
            if self
                .definitions_by_identity
                .contains_key(&artifact.identity())
            {
                return Err(D::DuplicateDefinition(artifact.identity()));
            }
            if let Some(previous) = artifact.predecessor() {
                if !self
                    .definitions_by_identity
                    .get(&previous.identity())
                    .and_then(|index| self.definitions.get(*index))
                    .is_some_and(|location| location.reference == previous)
                {
                    return Err(D::MissingDefinition(previous));
                }
            }
            self.definitions_by_identity
                .insert(artifact.identity(), self.definitions.len());
            self.definitions.push(RecordedDefinitionLocation {
                position,
                reference: artifact.reference(),
                branch: artifact.branch(),
            });
        }
        Ok(())
    }
    fn read_advances(&mut self, execution: &ExecutionRequest<'_, '_>) -> Result<(), D> {
        let end = self
            .original
            .frontier(SignalDurableJournal::Advances, execution)
            .map_err(D::Medium)?;
        for position in 1..=end {
            let bytes = self
                .original
                .read(SignalDurableJournal::Advances, position, execution)
                .map_err(D::Medium)?;
            let advance = SignalBranchAdvanceArtifact::readmit_bytes(execution, bytes.bytes())
                .map_err(D::Frame)?;
            let name_bytes = match advance.change() {
                SignalAuthoredBranchChange::Created { name, .. } => name.len(),
                SignalAuthoredBranchChange::SnapshotCaptured => self
                    .branches
                    .get(&advance.descriptor().branch())
                    .map_or(0, |branch| branch.name.len()),
                SignalAuthoredBranchChange::Advanced
                | SignalAuthoredBranchChange::Restored
                | SignalAuthoredBranchChange::Retired => 0,
            };
            if let Some(storage) = &mut self._storage {
                let total = storage
                    .bytes()
                    .checked_add(name_bytes as u64)
                    .ok_or(D::MetadataCapacity)?;
                storage.resize(total).map_err(D::Memory)?;
            }
            let descriptor = advance.descriptor();
            self.frontiers.observe_advance(advance.identity())?;
            self.frontiers.observe_descriptor(descriptor)?;
            if self.advances.contains_key(&advance.identity()) {
                return Err(D::DuplicateAdvance(advance.identity()));
            }
            self.advances.insert(
                advance.identity(),
                RecordedAdvanceLocation {
                    position,
                    digest: advance.digest(),
                },
            );
            self.definition_location(descriptor.definition(), descriptor.branch())?;
            match advance.change() {
                SignalAuthoredBranchChange::Created { name, parent } => {
                    if let Some(parent) = parent {
                        self.frontiers.observe_descriptor(parent)?;
                        self.definition_location(parent.definition(), parent.branch())?;
                        if !self
                            .branches
                            .get(&parent.branch())
                            .is_some_and(|branch| &branch.head == parent)
                        {
                            return Err(D::BranchChainMismatch(parent.branch()));
                        }
                    }
                    if self.branches.contains_key(&descriptor.branch()) {
                        return Err(D::DuplicateBranch(descriptor.branch()));
                    }
                    self.branches.insert(
                        descriptor.branch(),
                        RecoveredAuthoredBranch {
                            name: name.clone(),
                            parent: parent.clone(),
                            head: descriptor.clone(),
                            restored: false,
                        },
                    );
                }
                SignalAuthoredBranchChange::Advanced => self.advance_head(descriptor, false)?,
                SignalAuthoredBranchChange::SnapshotCaptured => {
                    let snapshot = descriptor.snapshot().ok_or(D::SnapshotRequired)?;
                    if self.snapshots.contains_key(&snapshot) {
                        return Err(D::DuplicateSnapshot(snapshot));
                    }
                    self.advance_head(descriptor, false)?;
                    let branch = self
                        .branches
                        .get(&descriptor.branch())
                        .expect("snapshot advanced its actual branch");
                    self.snapshots.insert(
                        snapshot,
                        RecoveredAuthoredSnapshot {
                            descriptor: descriptor.clone(),
                            name: branch.name.clone(),
                            parent: branch.parent.clone(),
                        },
                    );
                }
                SignalAuthoredBranchChange::Restored => {
                    let snapshot = descriptor.snapshot().ok_or(D::SnapshotRequired)?;
                    let source = self
                        .snapshots
                        .get(&snapshot)
                        .ok_or(D::MissingSnapshot(snapshot))?;
                    if source.descriptor.branch() != descriptor.branch() {
                        return Err(D::BranchChainMismatch(descriptor.branch()));
                    }
                    self.advance_head(descriptor, true)?;
                }
                SignalAuthoredBranchChange::Retired => {
                    let current = self
                        .branches
                        .get(&descriptor.branch())
                        .ok_or(D::MissingBranch(descriptor.branch()))?;
                    if &current.head != descriptor {
                        return Err(D::BranchChainMismatch(descriptor.branch()));
                    }
                    self.branches.remove(&descriptor.branch());
                    // Runtime retirement releases this branch's snapshot state.
                    // Original definition records still resolve through their
                    // immutable source index; historical bytes are not deleted.
                    self.snapshots
                        .retain(|_, snapshot| snapshot.descriptor.branch() != descriptor.branch());
                }
            }
        }
        Ok(())
    }
    fn advance_head(
        &mut self,
        descriptor: &SignalDurableBranchDescriptor,
        restored: bool,
    ) -> Result<(), D> {
        let branch = self
            .branches
            .get_mut(&descriptor.branch())
            .ok_or(D::MissingBranch(descriptor.branch()))?;
        if branch.head.cell() != descriptor.cell()
            || branch.head.generation().ordinal().checked_add(1)
                != Some(descriptor.generation().ordinal())
        {
            return Err(D::BranchChainMismatch(descriptor.branch()));
        }
        branch.head = descriptor.clone();
        branch.restored = restored;
        Ok(())
    }
    fn definition_location(
        &self,
        reference: SignalDefinitionTransitionReference,
        branch: DurableSignalBranchIdentity,
    ) -> Result<&RecordedDefinitionLocation, D> {
        self.definitions_by_identity
            .get(&reference.identity())
            .and_then(|index| self.definitions.get(*index))
            .filter(|row| row.reference == reference && row.branch == branch)
            .ok_or(D::MissingDefinition(reference))
    }
    pub(crate) fn read_definition(
        &self,
        descriptor: &SignalDurableBranchDescriptor,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDefinitionTransitionArtifact, D> {
        let location = self.definition_location(descriptor.definition(), descriptor.branch())?;
        self.read_location(location, execution)
    }
    pub(crate) fn read_location(
        &self,
        location: &RecordedDefinitionLocation,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDefinitionTransitionArtifact, D> {
        let bytes = self
            .original
            .read(
                SignalDurableJournal::Definitions,
                location.position,
                execution,
            )
            .map_err(D::Medium)?;
        let artifact =
            SignalDefinitionTransitionArtifact::readmit_owned(bytes).map_err(D::Frame)?;
        if artifact.reference() != location.reference || artifact.branch() != location.branch {
            return Err(D::MissingDefinition(location.reference));
        }
        Ok(artifact)
    }
}
