//! Resolve content through this admitted Branch root and its installed codec.
use super::{RelationalContentDenial as Denial, RelationalContentFrame};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_relational_contracts::internal_port::content::registry::RelationalContentFormatRegistryOwnerPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

use crate::branch::root::RelationalBranchRoot;
use crate::visibility::materialization::read_records::projection::VisibilityProjectionView;

impl AdmittedRelationalBranchBasis {
    pub fn read_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        format: CanonicalDigestId,
    ) -> Result<RelationalContentFrame, Denial> {
        read_root_content(&self.inner.root, execution, identity, format)
    }
}

impl VisibilityProjectionView {
    /// Read a content identity through the selected root and its installed codec.
    pub fn read_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        format: CanonicalDigestId,
    ) -> Result<RelationalContentFrame, Denial> {
        let root = self.selected_root().ok_or(Denial::MissingContent)?;
        read_root_content(root, execution, identity, format)
    }
}

fn read_root_content(
    root: &RelationalBranchRoot,
    execution: &ExecutionRequest<'_, '_>,
    identity: ContentRefId,
    format: CanonicalDigestId,
) -> Result<RelationalContentFrame, Denial> {
    super::control::run(execution, |control| {
        let schema = root.retained_schema_authority();
        control.checkpoint((*schema.registry().owner_content_formats()).lookup_bound())?;
        let codec = (*schema.registry().owner_content_formats()).get(format)?;
        root.retained_content().read(execution, identity, codec)
    })
}
