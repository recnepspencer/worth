//! Resolve content through this admitted Branch root and its installed codec.
use super::{RelationalContentDenial as Denial, RelationalContentFrame};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_relational_contracts::internal_port::content::registry::RelationalContentFormatRegistryOwnerPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

impl AdmittedRelationalBranchBasis {
    pub fn read_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        format: CanonicalDigestId,
    ) -> Result<RelationalContentFrame, Denial> {
        super::control::run(execution, |control| {
            let schema = self.inner.root.retained_schema_authority();
            control.checkpoint((*schema.registry().owner_content_formats()).lookup_bound())?;
            let codec = (*schema.registry().owner_content_formats()).get(format)?;
            self.inner
                .root
                .retained_content()
                .read(execution, identity, codec)
        })
    }
}
