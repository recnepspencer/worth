//! Read the immutable aspect plans admitted for one schema.
use crate::schema::data::{AspectContractPlanCatalog, LoweredAspectContractPlan};

pub trait AspectPlanSource {
    fn aspect_plan_catalog(&self) -> &AspectContractPlanCatalog;
    fn entity_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan>;
    fn relation_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan>;
}

impl AspectPlanSource for AspectContractPlanCatalog {
    fn aspect_plan_catalog(&self) -> &AspectContractPlanCatalog {
        self
    }
    fn entity_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.entity_plans.get(&kind_id)
    }
    fn relation_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.relation_plans.get(&kind_id)
    }
}
