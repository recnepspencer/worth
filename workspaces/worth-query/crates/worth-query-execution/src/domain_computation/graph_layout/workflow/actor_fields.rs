//! Where a workflow actor's three identity components live in a workflow record: installed
//! field locators, planned with the layout that owns them.
use worth_foundational::facade::AspectFieldLocator;

#[derive(Clone, Debug)]
pub struct WorkflowActorFields {
    pub(in crate::domain_computation) partition: AspectFieldLocator,
    pub(in crate::domain_computation) slot: AspectFieldLocator,
    pub(in crate::domain_computation) generation: AspectFieldLocator,
}

impl WorkflowActorFields {
    pub(in crate::domain_computation) fn planned(
        aspect: &str,
        role: &str,
    ) -> Result<
        Self,
        crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial,
    > {
        use crate::domain_computation::graph_layout::planned_field_locator;
        Ok(Self {
            partition: planned_field_locator(aspect, &format!("{role}-partition"))?,
            slot: planned_field_locator(aspect, &format!("{role}-slot"))?,
            generation: planned_field_locator(aspect, &format!("{role}-generation"))?,
        })
    }
}
