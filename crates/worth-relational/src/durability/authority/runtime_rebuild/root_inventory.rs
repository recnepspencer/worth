//! Exact source-produced root images retained by the one reconstruction owner.
use crate::branch::{RelationalBranchBasisDescriptor, RelationalBranchRoot};
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::{BranchId, CommitId};
use crate::runtime::RelationalRuntime;
use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

mod checkpoint_images;
mod world_handoff;

#[derive(Debug)]
enum RecoveredRootHold {
    Reconstructing(Arc<RelationalBranchRoot>),
    Owner(Weak<RelationalBranchRoot>),
    Empty {
        owner: Weak<RelationalBranchRoot>,
        schema: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
    },
    Checkpoint(std::sync::Mutex<Weak<RelationalBranchRoot>>),
}

impl RecoveredRootHold {
    fn root(&self) -> Option<Arc<RelationalBranchRoot>> {
        match self {
            Self::Reconstructing(root) => Some(root.clone()),
            Self::Owner(root) => root.upgrade(),
            Self::Checkpoint(root) => root
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .upgrade(),
            Self::Empty { owner, schema } => Some(owner.upgrade().unwrap_or_else(|| {
                RelationalBranchRoot::empty_with_schema_authority(schema.clone())
            })),
        }
    }
}

#[derive(Debug)]
struct RecoveredRootImage {
    root: RecoveredRootHold,
    observations: Vec<RelationalBranchBasisDescriptor>,
}

/// The existing reconstructive root inventory also preserves the actual cell
/// axes that selected each root. It acquires no observation/component pin until
/// a recorded subject is admitted for exposure.
#[derive(Debug, Default)]
pub(crate) struct RecoveredRootInventory {
    world_handoff: std::sync::Mutex<world_handoff::WorldSelectionHandoff>,
    roots: BTreeMap<CommitId, RecoveredRootImage>,
    directory: Option<Weak<crate::durability::data::RelationalCheckpointDirectory>>,
    empty: Vec<RecoveredRootImage>,
    _sharing_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RecoveredRootInventory {
    pub(super) fn capture(runtime: &RelationalRuntime) -> Result<Self, DurabilityError> {
        let mut inventory = Self::default();
        for branch_id in runtime.history.branch_ids_snapshot() {
            inventory.retain_current(runtime, &branch_id)?;
        }
        inventory.install_checkpoint_headers(runtime)?;
        Ok(inventory)
    }

    pub(in crate::durability::authority) fn publication_observations(
        &self,
        commit: CommitId,
    ) -> Option<&[RelationalBranchBasisDescriptor]> {
        self.roots
            .get(&commit)
            .map(|image| image.observations.as_slice())
    }

    pub(super) fn resolve(&self, commit_id: CommitId) -> Option<Arc<RelationalBranchRoot>> {
        self.roots
            .get(&commit_id)
            .and_then(|image| image.root.root())
    }

    /// The actual runtime retains materialized roots. After reconstruction,
    /// descriptive authority must not invisibly retain Home source/Serving
    /// ownership. Demanded admission upgrades a real root and acquires pins.
    pub(in crate::durability::authority) fn into_readmission(
        mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, crate::durability::data::RelationalRecoveryStop> {
        for image in self.roots.values_mut() {
            if let RecoveredRootHold::Reconstructing(root) = &image.root {
                image.root = RecoveredRootHold::Owner(Arc::downgrade(root));
            }
        }
        for image in &mut self.empty {
            if let RecoveredRootHold::Reconstructing(root) = &image.root {
                // A current Empty subject must retain the actual installed
                // head identity for later publication. Keep only a weak owner
                // share; historical Empty DATA can outlive that head through
                // its original immutable schema without retaining Home ports.
                image.root = RecoveredRootHold::Empty {
                    owner: Arc::downgrade(root),
                    schema: root.retained_schema_authority(),
                };
            }
        }
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| {
                crate::durability::data::RelationalRecoveryStop::Request(
                    crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                )
            })?
            .0
            .pad_to_align();
        self._sharing_memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| {
                crate::durability::data::RelationalRecoveryStop::Request(cause.into())
            })?;
        Ok(Arc::new(self))
    }

    /// The same admitted image inventory participates in Native source close.
    /// Weak image rows never keep a root alive or reconstruct one during close.
    pub(crate) fn source_retirement_root(
        &self,
        after: Option<CommitId>,
    ) -> Result<Option<(CommitId, Option<Arc<RelationalBranchRoot>>)>, ()> {
        use std::ops::Bound::{Excluded, Unbounded};
        let row = match after {
            Some(after) => self.roots.range((Excluded(after), Unbounded)).next(),
            None => self.roots.first_key_value(),
        };
        let Some((&commit, image)) = row else {
            return Ok(None);
        };
        let root = match &image.root {
            RecoveredRootHold::Reconstructing(root) => Some(root.clone()),
            RecoveredRootHold::Owner(root) => root.upgrade(),
            RecoveredRootHold::Checkpoint(root) => root.try_lock().map_err(|_| ())?.upgrade(),
            RecoveredRootHold::Empty {
                owner: _,
                schema: _,
            } => None,
        };
        Ok(Some((commit, root)))
    }

    pub(in crate::durability::authority) fn exact_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        recorded: &crate::branch::RecordedRelationalComponentReference,
    ) -> Result<
        Option<(Arc<RelationalBranchRoot>, RelationalBranchBasisDescriptor)>,
        crate::durability::data::RelationalRecoveryStop,
    > {
        let selected = match recorded.target() {
            crate::branch::RecordedRelationalComponentTarget::Empty => self
                .empty
                .iter()
                .find_map(|image| matching_observation(image, recorded)),
            crate::branch::RecordedRelationalComponentTarget::Canonical(target) => self
                .roots
                .get(&target.receipt().commit_id)
                .and_then(|image| matching_observation(image, recorded)),
        };
        let Some((image, descriptor)) = selected else {
            return Ok(None);
        };
        let root = match image.root.root() {
            Some(root) => root,
            None => self.materialize_checkpoint_image(execution, runtime, recorded, image)?,
        };
        let descriptor = checkpoint_images::bind_recorded_descriptor(descriptor, runtime, &root)?;
        Ok(Some((root, descriptor)))
    }

    pub(super) fn retain_current(
        &mut self,
        runtime: &RelationalRuntime,
        branch_id: &BranchId,
    ) -> Result<(), DurabilityError> {
        let Some(cell) = runtime.history.branch_cell(branch_id) else {
            return Ok(());
        };
        let root = cell.root().unwrap_or_else(|| {
            RelationalBranchRoot::empty_with_schema(
                &runtime.config.schema.registry,
                crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
            )
        });
        let descriptor = crate::branch::descriptor_for_cell(&cell, &root)
            .map_err(|cause| corrupt(format!("recovered root observation: {cause:?}")))?;
        if matches!(
            cell.observation().target(),
            worth_foundational::FoundationalBranchTarget::Empty
        ) {
            if root.id() != 0 || root.descriptor().is_some() {
                return Err(corrupt("recovered Empty subject owns a committed root"));
            }
            if !self
                .empty
                .iter()
                .any(|image| image.observations.contains(&descriptor))
            {
                self.empty.push(RecoveredRootImage {
                    root: RecoveredRootHold::Reconstructing(root),
                    observations: vec![descriptor],
                });
            }
            return Ok(());
        }
        let commit_id = root
            .commit_id()
            .ok_or_else(|| corrupt("recovered canonical subject has no committed root"))?;
        match self.roots.entry(commit_id) {
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let image = entry.get_mut();
                if image
                    .root
                    .root()
                    .is_none_or(|existing| !Arc::ptr_eq(&existing, &root))
                {
                    return Err(corrupt(format!(
                        "recovery produced competing roots for commit {}",
                        commit_id.0
                    )));
                }
                if !image.observations.contains(&descriptor) {
                    image.observations.push(descriptor);
                }
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(RecoveredRootImage {
                    root: RecoveredRootHold::Reconstructing(root),
                    observations: vec![descriptor],
                });
            }
        }
        Ok(())
    }
}

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}

fn matching_observation<'a>(
    image: &'a RecoveredRootImage,
    recorded: &crate::branch::RecordedRelationalComponentReference,
) -> Option<(&'a RecoveredRootImage, &'a RelationalBranchBasisDescriptor)> {
    image
        .observations
        .iter()
        .find(|descriptor| {
            descriptor.in_canonical_frame().ok().as_ref() == Some(recorded.descriptor())
        })
        .map(|descriptor| (image, descriptor))
}
