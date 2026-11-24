//! Reconstructive checkpoint schema records share their canonical carrier backing.
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::schema_carrier::{RecordedBranchSchemaImage, RecordedBranchSchemaImageFixturePort as _};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurableBranchRootSchemaImage(RecordedBranchSchemaImage);

impl DurableBranchRootSchemaImage {
    pub(crate) fn capture(
        authority: &RelationalBranchRootSchemaAuthority,
    ) -> Result<Self, rmp_serde::encode::Error> {
        RecordedBranchSchemaImage::capture(authority.schema_image_source()).map(Self)
    }

    pub(crate) fn recorded_for_decode(&mut self) -> &mut RecordedBranchSchemaImage {
        &mut self.0
    }

    pub fn corrupt_schema_root_for_test(&mut self) {
        self.0.corrupt_schema_root_for_test();
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
