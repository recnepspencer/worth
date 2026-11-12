//! Replay exact branch originals and retain their physical source points.
use super::*;
use crate::durability::{SignalAuthoredBranchChange, SignalBranchAdvanceArtifact};
impl SignalOwnerSourceReadmission {
    pub(super) fn read_advances(&mut self, execution: &ExecutionRequest<'_, '_>) -> Result<(), D> {
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
                            .is_some_and(|branch| branch.head.descriptor() == parent)
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
                            head: crate::durability::SignalRecordedSourcePoint::original(
                                &advance,
                                std::num::NonZeroU64::new(position)
                                    .expect("original journal position"),
                            ),
                            restored: false,
                        },
                    );
                }
                SignalAuthoredBranchChange::Advanced => {
                    self.advance_head(&advance, position, false)?
                }
                SignalAuthoredBranchChange::SnapshotCaptured => {
                    let snapshot = descriptor.snapshot().ok_or(D::SnapshotRequired)?;
                    if self.snapshots.contains_key(&snapshot) {
                        return Err(D::DuplicateSnapshot(snapshot));
                    }
                    self.advance_head(&advance, position, false)?;
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
                    self.advance_head(&advance, position, true)?;
                }
                SignalAuthoredBranchChange::Retired => {
                    let current = self
                        .branches
                        .get(&descriptor.branch())
                        .ok_or(D::MissingBranch(descriptor.branch()))?;
                    if current.head.descriptor() != descriptor {
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
        advance: &SignalBranchAdvanceArtifact,
        position: u64,
        restored: bool,
    ) -> Result<(), D> {
        let descriptor = advance.descriptor();
        let branch = self
            .branches
            .get_mut(&descriptor.branch())
            .ok_or(D::MissingBranch(descriptor.branch()))?;
        if branch.head.descriptor().cell() != descriptor.cell()
            || branch
                .head
                .descriptor()
                .generation()
                .ordinal()
                .checked_add(1)
                != Some(descriptor.generation().ordinal())
        {
            return Err(D::BranchChainMismatch(descriptor.branch()));
        }
        branch.head = crate::durability::SignalRecordedSourcePoint::original(
            advance,
            std::num::NonZeroU64::new(position).expect("original journal position"),
        );
        branch.restored = restored;
        Ok(())
    }
}
