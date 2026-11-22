//! Pure delegation activation binding data.
use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_schema::ApplicationOperationProgramTarget;
use worth_relational::facade::identity::{EntityId, KindId};

pub mod program;
pub use program::WorthQueryDelegationActivationEffect;

pub struct WorthQueryDelegationActivationBinding {
    pub proposal_identity: [u8; 32],
    pub required_program_targets: Vec<ApplicationOperationProgramTarget>,
    pub child_kind: KindId,
    pub child_key: String,
    pub fields: BTreeMap<AspectFieldLocator, AspectValue>,
    pub parent_relation: KindId,
    pub grantor_relation: KindId,
    pub grantee_relation: KindId,
    pub resource_relation: KindId,
    pub related_relation: Option<KindId>,
    pub parent: EntityId,
    pub grantor: EntityId,
    pub grantee: EntityId,
    pub resource: EntityId,
    pub related: Option<EntityId>,
    pub activation_context: Vec<(KindId, EntityId)>,
}

impl WorthQueryDelegationActivationBinding {
    pub const fn proposal_identity(&self) -> &[u8; 32] {
        &self.proposal_identity
    }
}
