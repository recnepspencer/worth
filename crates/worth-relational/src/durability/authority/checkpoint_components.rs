//! Exact current cells and external obligations, retained by the actual capture.
use crate::branch::{
    RelationalBranchBasisDescriptor, RelationalBranchRoot, RelationalBranchRootCheckpoint,
};
use crate::durability::data::{
    DurabilityError, DurableBranchRootSchemaImage, RecoveryFailureClass,
    RelationalCheckpointEmptyComponent,
};
use crate::history::data::CommitId;
use crate::runtime::RelationalRuntime;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) struct CapturedComponents {
    pub(super) roots: Vec<RelationalBranchRootCheckpoint>,
    pub(super) canonical: BTreeMap<CommitId, Vec<RelationalBranchBasisDescriptor>>,
    pub(super) empty: Vec<RelationalCheckpointEmptyComponent>,
    pub(super) empty_schemas: Vec<DurableBranchRootSchemaImage>,
    // The same actual observations outlive the original external lease release.
    pub(super) held: Vec<crate::branch::AdmittedRelationalBranchBasis>,
}

pub(super) fn capture(runtime: &RelationalRuntime) -> Result<CapturedComponents, DurabilityError> {
    let held = runtime.history.external_checkpoint_components();
    let mut observations = BTreeMap::new();
    for branch in runtime.history.branch_ids_snapshot() {
        let cell = runtime
            .history
            .branch_cell(&branch)
            .ok_or_else(|| corrupt("current branch disappeared"))?;
        let root = cell
            .root()
            .ok_or_else(|| corrupt("current branch has no root"))?;
        let descriptor = crate::branch::descriptor_for_cell(&cell, &root)
            .map_err(|cause| corrupt(format!("component descriptor: {cause:?}")))?;
        insert(&mut observations, descriptor, root)?;
    }
    for basis in &held {
        insert(
            &mut observations,
            basis.descriptor().clone(),
            basis.inner.root.clone(),
        )?;
    }
    let mut roots = BTreeMap::new();
    let mut canonical = BTreeMap::<CommitId, Vec<_>>::new();
    let mut empty = Vec::new();
    let mut schemas = BTreeMap::new();
    for (_, (descriptor, root)) in observations {
        match descriptor.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => {
                if root.id() != 0 || root.descriptor().is_some() {
                    return Err(corrupt("Empty component has canonical root"));
                }
                let image = DurableBranchRootSchemaImage::capture(root.schema_authority())
                    .map_err(|cause| corrupt(cause.to_string()))?;
                let digest = image.carrier_digest();
                schemas.entry(digest).or_insert(image);
                empty.push(RelationalCheckpointEmptyComponent {
                    descriptor,
                    schema_carrier_digest: digest,
                });
            }
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                let commit = CommitId(target.selected_commit_id());
                if root.commit_id() != Some(commit) || root.descriptor() != Some(target.roots()) {
                    return Err(corrupt("component root disagrees with actual observation"));
                }
                if let Some(prior) = roots.insert(commit, root.clone()) {
                    if !Arc::ptr_eq(&prior, &root) {
                        return Err(corrupt("competing component roots"));
                    }
                }
                canonical.entry(commit).or_default().push(descriptor);
            }
        }
    }
    Ok(CapturedComponents {
        roots: roots
            .into_iter()
            .map(|(id, root)| RelationalBranchRootCheckpoint::new(id, root))
            .collect(),
        canonical,
        empty,
        empty_schemas: schemas.into_values().collect(),
        held,
    })
}

fn insert(
    observations: &mut BTreeMap<
        Vec<u8>,
        (RelationalBranchBasisDescriptor, Arc<RelationalBranchRoot>),
    >,
    descriptor: RelationalBranchBasisDescriptor,
    root: Arc<RelationalBranchRoot>,
) -> Result<(), DurabilityError> {
    let descriptor = descriptor
        .in_canonical_frame()
        .map_err(|cause| corrupt(format!("component frame: {cause:?}")))?;
    let key = rmp_serde::to_vec_named(&descriptor).map_err(|cause| corrupt(cause.to_string()))?;
    if let Some((_, prior)) = observations.insert(key, (descriptor, root.clone())) {
        if !Arc::ptr_eq(&prior, &root) {
            return Err(corrupt("one component selects competing roots"));
        }
    }
    Ok(())
}
fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
