//! Exact source-produced root images retained by the one reconstruction owner.
use crate::branch::{RelationalBranchBasisDescriptor, RelationalBranchRoot};
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::{BranchId, CommitId};
use crate::runtime::RelationalRuntime;
use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

#[derive(Debug)]
enum RecoveredRootHold {
    Reconstructing(Arc<RelationalBranchRoot>),
    Owner(Weak<RelationalBranchRoot>),
    Empty(Arc<crate::branch::RelationalBranchRootSchemaAuthority>),
}

impl RecoveredRootHold {
    fn root(&self) -> Option<Arc<RelationalBranchRoot>> {
        match self {
            Self::Reconstructing(root) => Some(root.clone()),
            Self::Owner(root) => root.upgrade(),
            Self::Empty(schema) => Some(RelationalBranchRoot::empty_with_schema(
                schema.registry(),
                schema.descriptor_semantics_version(),
            )),
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
pub(in crate::durability::authority) struct RecoveredRootInventory {
    roots: BTreeMap<CommitId, RecoveredRootImage>,
    empty: Vec<RecoveredRootImage>,
}

impl RecoveredRootInventory {
    pub(super) fn capture(runtime: &RelationalRuntime) -> Result<Self, DurabilityError> {
        let mut inventory = Self::default();
        for branch_id in runtime.history.branch_ids_snapshot() {
            inventory.retain_current(runtime, &branch_id)?;
        }
        Ok(inventory)
    }

    pub(super) fn resolve(&self, commit_id: CommitId) -> Option<Arc<RelationalBranchRoot>> {
        self.roots
            .get(&commit_id)
            .and_then(|image| image.root.root())
    }

    /// The actual runtime retains materialized roots. After reconstruction,
    /// descriptive authority must not invisibly retain Home source/Serving
    /// ownership. Demanded admission upgrades a real root and acquires pins.
    pub(in crate::durability::authority) fn into_readmission(mut self) -> Self {
        for image in self.roots.values_mut() {
            if let RecoveredRootHold::Reconstructing(root) = &image.root {
                image.root = RecoveredRootHold::Owner(Arc::downgrade(root));
            }
        }
        for image in &mut self.empty {
            if let RecoveredRootHold::Reconstructing(root) = &image.root {
                image.root = RecoveredRootHold::Empty(root.retained_schema_authority());
            }
        }
        self
    }

    pub(in crate::durability::authority) fn exact_image(
        &self,
        recorded: &crate::branch::RecordedRelationalComponentReference,
    ) -> Option<(Arc<RelationalBranchRoot>, RelationalBranchBasisDescriptor)> {
        let find = |image: &RecoveredRootImage| {
            image
                .observations
                .iter()
                .find(|descriptor| {
                    descriptor.in_canonical_frame().ok().as_ref() == Some(recorded.descriptor())
                })
                .and_then(|descriptor| image.root.root().map(|root| (root, descriptor.clone())))
        };
        match recorded.target() {
            crate::branch::RecordedRelationalComponentTarget::Empty => {
                self.empty.iter().find_map(find)
            }
            crate::branch::RecordedRelationalComponentTarget::Canonical(target) => {
                self.roots.get(&target.receipt().commit_id).and_then(find)
            }
        }
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
