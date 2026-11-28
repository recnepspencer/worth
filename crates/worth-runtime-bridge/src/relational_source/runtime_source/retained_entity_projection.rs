use crate::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSourceError, TruthBranchIdentity,
    TruthSnapshotIdentity,
};
use worth_foundational::facade::AuthoritativeRecordAspectState;

use super::RuntimeBridgeRelationalSource;
use worth_relational::facade::transactions::RecordRef;

impl RuntimeBridgeRelationalSource {
    /// Project one entity's authoritative aspect state from an exact retained
    /// observation without exposing the owning Relational runtime.
    #[doc(hidden)]
    pub fn read_retained_entity_aspect_state(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &TruthSnapshotIdentity,
        branch: &TruthBranchIdentity,
        record: RelationalBridgeRecordIdentityParts,
    ) -> Result<Option<AuthoritativeRecordAspectState>, RelationalBridgeSourceError> {
        let observation = self.observation_bindings.resolve(snapshot)?;
        if branch.branch_id() != Some(observation.branch_id()) {
            return Err(RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::Binding(
                    crate::adapter::BridgeSourceBindingDenial::TruthBranchMismatch,
                ),
                "retained relational observation belongs to a different truth branch",
            ));
        }
        if !self.admits_relational_partition(record.partition_id()) {
            return Err(RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::Binding(
                    crate::adapter::BridgeSourceBindingDenial::PartitionAuthorityMismatch,
                ),
                "retained relational entity projection is outside the source partition authority",
            ));
        }
        let RecordRef::Entity(entity_id) = record.record_ref() else {
            return Err(RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::Binding(
                    crate::adapter::BridgeSourceBindingDenial::EntityIdentityRequired,
                ),
                "retained relational entity projection requires an entity identity",
            ));
        };

        self.runtime
            .with_runtime(|runtime| {
                runtime.entity_record_at_observation(
                    execution,
                    observation.observation(),
                    entity_id,
                )
            })
            .map(|record| record.and_then(|record| record.authoritative_aspect_state))
            .map_err(|denial| {
                RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ObservationRead(denial),
                    format!("retained relational entity projection refused: {denial:?}"),
                )
            })
    }
}
