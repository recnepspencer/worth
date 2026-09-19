use super::{RuntimeArtifactHot, RuntimeArtifactState, RuntimeArtifactWarm};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for RuntimeArtifactState {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { hot, warm } = self;
        hot.retained_heap_charge(work)?
            .checked_add(warm.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for RuntimeArtifactHot {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            output_hash: _,
            output_change: _,
            recomputed: _,
            dependency_count: _,
            meaningful_input_changes: _,
            changed_partition_count: _,
            propagation_suppressed: _,
            changed_scopes,
        } = self;
        changed_scopes.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for RuntimeArtifactWarm {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            output_identity,
            continuity_token,
            memoized_origin: _,
            reuse_basis,
            reuse_origin: _,
            reuse_boundary_authority,
            lineage_artifact_id: _,
            merge_authority,
        } = self;
        let super::super::authority::ArtifactMergeAuthority {
            authority_class: _,
            adoptability: _,
        } = merge_authority;
        output_identity
            .retained_heap_charge(work)?
            .checked_add(continuity_token.retained_heap_charge(work)?)?
            .checked_add(reuse_basis.retained_heap_charge(work)?)?
            .checked_add(reuse_boundary_authority.retained_heap_charge(work)?)
    }
}
