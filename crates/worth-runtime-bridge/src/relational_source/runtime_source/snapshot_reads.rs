use crate::facade::{
    RelationalBridgeSourceError, SnapshotReadSource, TruthSnapshotIdentity, TruthSnapshotReader,
};

use super::RuntimeBridgeRelationalSource;
use crate::relational_source::snapshot_reading::RuntimePublicationSnapshotReader;

impl SnapshotReadSource for RuntimeBridgeRelationalSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        let observation = self.observation_bindings.resolve(identity)?;
        Ok(Box::new(
            RuntimePublicationSnapshotReader::for_observation_authority(
                self.runtime.clone(),
                identity.clone(),
                observation.observation().clone(),
                self.partition
                    .as_ref()
                    .map(|partition| partition.relational),
            ),
        ))
    }
}
