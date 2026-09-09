use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use serde::{Deserialize, Serialize};

use super::{SignalSchemaId, SignalSchemaName, SignalSchemaVersion};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSchemaBinding {
    schema_id: SignalSchemaId,
    semantic_name: SignalSchemaName,
    version: SignalSchemaVersion,
    descriptor_digest: String,
}

impl SignalSchemaBinding {
    pub fn new(
        schema_id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        descriptor_digest: impl Into<String>,
    ) -> Self {
        Self {
            schema_id,
            semantic_name,
            version,
            descriptor_digest: descriptor_digest.into(),
        }
    }

    pub fn schema_id(&self) -> SignalSchemaId {
        self.schema_id
    }

    pub fn semantic_name(&self) -> &SignalSchemaName {
        &self.semantic_name
    }

    pub fn version(&self) -> SignalSchemaVersion {
        self.version
    }

    pub fn descriptor_digest(&self) -> &str {
        &self.descriptor_digest
    }
}

impl RetainedStorageMeasurement for SignalSchemaBinding {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            semantic_name,
            descriptor_digest,
            schema_id: _,
            version: _,
        } = self;
        Ok(Charge::ZERO
            .checked_add(semantic_name.retained_heap_charge(work)?)?
            .checked_add(descriptor_digest.retained_heap_charge(work)?)?)
    }
}
