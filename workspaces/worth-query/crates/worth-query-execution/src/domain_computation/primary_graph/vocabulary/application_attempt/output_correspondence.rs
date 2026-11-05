//! Moved whole from `domain_computation::primary_graph::application_attempt::effect_program::output_correspondence`: the items below and every impl they own.
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryCheckpointOutputRole {
    pub(in crate::domain_computation::primary_graph) role: String,
    pub(in crate::domain_computation::primary_graph) posture: WorthQueryApplicationOutputPosture,
    pub(in crate::domain_computation::primary_graph) entity_name: String,
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
}

impl WorthQueryCheckpointOutputRole {
    #[doc(hidden)]
    pub fn entity_mut_for_test(&mut self) -> &mut EntityId {
        &mut self.entity
    }
}
