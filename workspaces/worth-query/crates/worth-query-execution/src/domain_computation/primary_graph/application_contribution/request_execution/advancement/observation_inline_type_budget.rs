//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryAdvancementIdentityForTest>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/request_execution/advancement/observation.rs", "WorthQueryAdvancementIdentityForTest", 128 * 1024);
        owner.field::<String>("Producer.producer");
        owner.field::<[u8; 32]>("Producer.query");
        owner.field::<[u8; 32]>("Producer.parameters");
        owner.field::<worth_relational::facade::identity::EntityId>("Producer.root");
        owner.field::<[u8; 32]>("Producer.source");
        owner.field::<[u8; 32]>("Mutation.operation");
        owner.field::<[u8; 32]>("Mutation.key");
        owner.field::<[u8; 32]>("Mutation.intent");
    }
}
