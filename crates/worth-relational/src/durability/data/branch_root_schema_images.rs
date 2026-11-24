//! Reconstructive checkpoint schema records share their canonical carrier backing.
use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::history::data::canonical_commit_envelope::schema_carrier::RecordedBranchSchemaImage;
use serde::{Deserialize, Serialize};

pub(crate) use crate::history::data::canonical_commit_envelope::schema_carrier::{
    DurableEntitySchemaRegistration, DurableSchemaDeclarationMemory,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct DurableBranchRootSchemaImage(RecordedBranchSchemaImage);

impl DurableBranchRootSchemaImage {
    pub(crate) fn capture(
        authority: &RelationalBranchRootSchemaAuthority,
    ) -> Result<Self, rmp_serde::encode::Error> {
        RecordedBranchSchemaImage::capture(authority.schema_image_source()).map(Self)
    }

    pub(crate) fn recorded_for_decode(&mut self) -> &mut RecordedBranchSchemaImage {
        &mut self.0
    }

    #[cfg(test)]
    pub(crate) fn corrupt_schema_root_for_test(&mut self) {
        self.0.corrupt_schema_root();
    }
}

impl From<RecordedBranchSchemaImage> for DurableBranchRootSchemaImage {
    fn from(recorded: RecordedBranchSchemaImage) -> Self {
        Self(recorded)
    }
}

impl std::ops::Deref for DurableBranchRootSchemaImage {
    type Target = RecordedBranchSchemaImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
