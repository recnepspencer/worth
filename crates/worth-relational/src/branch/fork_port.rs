use std::sync::Arc;

use crate::durability::medium::RelationalDurableMediumBinding;
use crate::runtime::{RelationalForkOwnerBinding, RelationalRuntimeOwnerBinding};

/// Cloneable branch-fork service bound to one live relational runtime owner.
/// It publishes each fork to the runtime's durable medium, when the runtime
/// owns one, before the target exists.
#[derive(Debug, Clone)]
pub struct RelationalForkPort {
    pub(super) runtime_instance_id: u64,
    pub(super) lifecycle: RelationalRuntimeOwnerBinding,
    pub(super) owner: RelationalForkOwnerBinding,
    pub(super) medium: Option<Arc<RelationalDurableMediumBinding>>,
    pub(super) indexes: crate::runtime::IndexDefinitionReadBinding,
    pub(super) publication: crate::runtime::RelationalRuntimePublicationBinding,
}

impl RelationalForkPort {
    pub(crate) fn new(
        runtime_instance_id: u64,
        lifecycle: RelationalRuntimeOwnerBinding,
        owner: RelationalForkOwnerBinding,
        medium: Option<Arc<RelationalDurableMediumBinding>>,
        publication: crate::runtime::RelationalRuntimePublicationBinding,
        indexes: crate::runtime::IndexDefinitionReadBinding,
    ) -> Self {
        Self {
            runtime_instance_id,
            lifecycle,
            owner,
            medium,
            publication,
            indexes,
        }
    }
}
