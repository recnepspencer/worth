use worth_relational::facade::identity::EntityId;

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
/// A typed actor requirement after the advancing actor was denied.
/// No live head is observed: mutation denial grants no read authority.
#[derive(Clone, Debug)]
pub struct RequiredWorkflowActor {
    instance: EntityId,
    denial: WorthQueryOperationAuthorizationDenial,
}

impl RequiredWorkflowActor {
    pub(in crate::domain_computation::primary_graph) fn new(
        instance: EntityId,
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self { instance, denial }
    }

    pub const fn instance(&self) -> EntityId {
        self.instance
    }

    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }
}
