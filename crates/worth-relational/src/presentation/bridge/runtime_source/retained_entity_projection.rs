use worth_foundational::facade::AuthoritativeRecordAspectState;
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSourceError, TruthBranchIdentity,
    TruthSnapshotIdentity,
};

use super::RuntimeBridgeRelationalSource;
use crate::facade::transactions::RecordRef;
use crate::presentation::bridge::identities::record_ref_from_identity_parts;

impl RuntimeBridgeRelationalSource {
    /// Project one entity's authoritative aspect state from an exact retained
    /// observation without exposing the owning Relational runtime.
    #[doc(hidden)]
    pub fn read_retained_entity_aspect_state(
        &self,
        snapshot: &TruthSnapshotIdentity,
        branch: &TruthBranchIdentity,
        record: RelationalBridgeRecordIdentityParts,
    ) -> Result<Option<AuthoritativeRecordAspectState>, RelationalBridgeSourceError> {
        let observation = self.observation_bindings.resolve(snapshot)?;
        let observed_branch =
            TruthBranchIdentity::from_relational_branch_id(observation.branch_id().0.clone());
        if &observed_branch != branch {
            return Err(RelationalBridgeSourceError::new(
                "retained relational observation belongs to a different truth branch",
            ));
        }
        if !self.admits_relational_partition(record.partition_id()) {
            return Err(RelationalBridgeSourceError::new(
                "retained relational entity projection is outside the source partition authority",
            ));
        }
        let RecordRef::Entity(entity_id) = record_ref_from_identity_parts(record)
            .map_err(|error| RelationalBridgeSourceError::new(error.to_string()))?
        else {
            return Err(RelationalBridgeSourceError::new(
                "retained relational entity projection requires an entity identity",
            ));
        };

        Ok(self.runtime.with_runtime(|runtime| {
            runtime
                .entity_record_at_observation(observation.observation(), entity_id)
                .and_then(|record| record.authoritative_aspect_state)
        }))
    }
}
