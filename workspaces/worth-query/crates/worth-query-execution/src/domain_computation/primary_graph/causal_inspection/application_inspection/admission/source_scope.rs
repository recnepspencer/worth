//! Borrow the issued source identity through the final consuming inspection check.
use super::*;
use crate::domain_computation::primary_graph::{
    entity_resolution::WorthQueryEntityResolutionTruth, WorthQueryEntityResolutionDenial,
};
use worth_relational::facade::identity::EntityId;

pub(super) trait InspectionSourceScope {
    fn entity_id(&self) -> EntityId;
    fn validate_currentness(
        &self,
        truth: &WorthQueryEntityResolutionTruth<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryEntityResolutionDenial>;
}
impl<Schema, Entity> InspectionSourceScope for WorthQueryApplicationEntityIdentity<Schema, Entity> {
    fn entity_id(&self) -> EntityId {
        self.entity_id()
    }
    fn validate_currentness(
        &self,
        truth: &WorthQueryEntityResolutionTruth<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        truth.validate_entity_freshness(execution, self)
    }
}
