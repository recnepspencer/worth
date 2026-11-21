//! Pure planned field locators for recorded workflow actor coordinates.

use worth_foundational::facade::AspectFieldLocator;

use crate::denial::WorthQuerySchemaLayoutDenial;
use crate::schema_layout::planned_field_locator;

#[derive(Clone, Debug)]
pub struct WorkflowActorFields {
    pub partition: AspectFieldLocator,
    pub slot: AspectFieldLocator,
    pub generation: AspectFieldLocator,
}

impl WorkflowActorFields {
    pub fn planned(aspect: &str, role: &str) -> Result<Self, WorthQuerySchemaLayoutDenial> {
        Ok(Self {
            partition: planned_field_locator(aspect, &format!("{role}-partition"))?,
            slot: planned_field_locator(aspect, &format!("{role}-slot"))?,
            generation: planned_field_locator(aspect, &format!("{role}-generation"))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowEndingActorFields {
    pub payload: AspectFieldLocator,
}

impl WorkflowEndingActorFields {
    pub fn planned() -> Result<Self, WorthQuerySchemaLayoutDenial> {
        Ok(Self {
            payload: planned_field_locator("workflow-instance", "ending-actor")?,
        })
    }
}
