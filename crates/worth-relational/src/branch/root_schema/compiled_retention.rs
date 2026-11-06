//! Child shares retain their original schema-compilation grant independently.
use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};
use std::{ops::Deref, sync::Arc};
use worth_execution::ExecutionMemoryReservation;

/// Its Arc allocation is included in the schema compilation quote. It owns no
/// runtime port, request loan, filesystem lease or read authority.
#[derive(Debug)]
pub(super) struct SchemaCompilationFunding {
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}

#[derive(Clone, Debug)]
pub(crate) struct RetainedRelationalSchemaRegistry {
    registry: Arc<RelationalSchemaRegistry>,
    _funding: Option<Arc<SchemaCompilationFunding>>,
}
impl RetainedRelationalSchemaRegistry {
    pub(super) fn from_schema(
        registry: Arc<RelationalSchemaRegistry>,
        funding: Option<Arc<SchemaCompilationFunding>>,
    ) -> Self {
        Self {
            registry,
            _funding: funding,
        }
    }

    pub(crate) fn from_configuration(registry: Arc<RelationalSchemaRegistry>) -> Self {
        Self {
            registry,
            _funding: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn same_allocation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.registry, &other.registry)
    }
}
impl Deref for RetainedRelationalSchemaRegistry {
    type Target = RelationalSchemaRegistry;
    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}
impl AsRef<RelationalSchemaRegistry> for RetainedRelationalSchemaRegistry {
    fn as_ref(&self) -> &RelationalSchemaRegistry {
        &self.registry
    }
}
impl PartialEq for RetainedRelationalSchemaRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry
    }
}
impl Eq for RetainedRelationalSchemaRegistry {}

#[derive(Clone, Debug)]
pub(crate) struct RetainedRelationalAspectPlans {
    plans: Arc<AspectContractPlanCatalog>,
    _funding: Option<Arc<SchemaCompilationFunding>>,
}
impl RetainedRelationalAspectPlans {
    pub(super) fn from_schema(
        plans: Arc<AspectContractPlanCatalog>,
        funding: Option<Arc<SchemaCompilationFunding>>,
    ) -> Self {
        Self {
            plans,
            _funding: funding,
        }
    }

    pub(crate) fn from_configuration(plans: AspectContractPlanCatalog) -> Self {
        Self {
            plans: Arc::new(plans),
            _funding: None,
        }
    }
}
impl Deref for RetainedRelationalAspectPlans {
    type Target = AspectContractPlanCatalog;
    fn deref(&self) -> &Self::Target {
        &self.plans
    }
}
impl AsRef<AspectContractPlanCatalog> for RetainedRelationalAspectPlans {
    fn as_ref(&self) -> &AspectContractPlanCatalog {
        &self.plans
    }
}
impl Default for RetainedRelationalAspectPlans {
    fn default() -> Self {
        Self::from_configuration(AspectContractPlanCatalog::empty())
    }
}
