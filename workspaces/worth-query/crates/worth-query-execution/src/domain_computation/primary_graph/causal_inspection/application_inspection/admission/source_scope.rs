//! Borrow the issued source identity through the final consuming inspection check.
use super::*;
use crate::domain_computation::primary_graph::{
    entity_resolution::WorthQueryEntityResolutionTruth, WorthQueryEntityResolutionDenial,
};
use worth_relational::facade::identity::EntityId;

pub(super) trait InspectionSourceScope {
    fn entity_id(&self) -> EntityId;
    fn entity_kind(&self) -> worth_relational::facade::identity::KindId;
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
    fn entity_kind(&self) -> worth_relational::facade::identity::KindId {
        self.entity_kind()
    }
    fn validate_currentness(
        &self,
        truth: &WorthQueryEntityResolutionTruth<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        truth.validate_entity_freshness(execution, self)
    }
}
