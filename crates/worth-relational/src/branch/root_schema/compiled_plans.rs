//! Standalone installation uses the same plan compiler and intrinsic custody.
use super::{
    aspect_plan_quote,
    compiled_retention::{RetainedRelationalAspectPlans, SchemaCompilationFunding},
};
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::schema::{data::RelationalSchemaRegistry, SchemaCompilationControl};
use std::sync::Arc;
impl RetainedRelationalAspectPlans {
    pub(crate) fn compile(
        registry: &RelationalSchemaRegistry,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, Denial> {
        let funding = match control {
            SchemaCompilationControl::Configuration => None,
            SchemaCompilationControl::Request(_) => {
                let bytes = aspect_plan_quote::retained_bytes(registry, control)?
                    .checked_add(aspect_plan_quote::arc_bytes::<SchemaCompilationFunding>()?)
                    .ok_or(Denial::MemoryOverflow)?;
                let memory = control.reserve(bytes)?;
                control.checkpoint()?;
                Some(Arc::new(SchemaCompilationFunding { _memory: memory }))
            }
        };
        let plans = crate::schema::lower_aspect_plans_for_compilation(registry, control)?;
        control.checkpoint()?;
        Ok(Self::from_schema(Arc::new(plans), funding))
    }
}
