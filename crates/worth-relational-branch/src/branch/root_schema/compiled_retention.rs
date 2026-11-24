//! Child shares retain their original schema-compilation grant independently.
use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};
use std::{ops::Deref, sync::Arc};
use worth_execution::ExecutionMemoryReservation;
use worth_relational_contracts::internal_port::content::registry::RelationalContentFormatRegistryOwnerPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

/// Its Arc allocation is included in the schema compilation quote. It owns no
/// runtime port, request loan, filesystem lease or read authority.
#[derive(Debug)]
pub(super) struct SchemaCompilationFunding {
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}

#[derive(Clone, Debug)]
pub struct RetainedRelationalSchemaRegistry {
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

    pub fn from_configuration(registry: Arc<RelationalSchemaRegistry>) -> Self {
        Self {
            registry,
            _funding: None,
        }
    }

    pub fn copy_for_request(
        registry: &RelationalSchemaRegistry,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::branch::root::RelationalBranchRootCaptureDenial> {
        use super::compilation_quote::{add, arc};
        let control = crate::schema::SchemaCompilationControl::Request(*request);
        let mut bytes = super::registry_quote::retained(registry, control)?;
        add(&mut bytes, arc::<RelationalSchemaRegistry>()?)?;
        add(&mut bytes, arc::<SchemaCompilationFunding>()?)?;
        let memory = request.reserve_retained_memory(bytes).map_err(|cause| {
            crate::branch::root::RelationalBranchRootCaptureDenial::Request(cause.into())
        })?;
        control
            .checkpoint()
            .map_err(crate::branch::root::RelationalBranchRootCaptureDenial::Request)?;
        let mut copied = registry.clone();
        let content = (*registry.owner_content_formats())
            .snapshot_for_compilation(control)
            .map_err(crate::branch::root::RelationalBranchRootCaptureDenial::Request)?;
        (*copied.owner_content_formats_mut()).bind_carried_snapshot(&content);
        Ok(Self {
            registry: Arc::new(copied),
            _funding: Some(Arc::new(SchemaCompilationFunding { _memory: memory })),
        })
    }

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
pub struct RetainedRelationalAspectPlans {
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

    pub fn from_configuration(plans: AspectContractPlanCatalog) -> Self {
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

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait RetainedRelationalSchemaRegistryFixturePort: Sized {
    fn same_allocation_for_test(&self, other: &Self) -> bool;
}

impl RetainedRelationalSchemaRegistryFixturePort for RetainedRelationalSchemaRegistry {
    fn same_allocation_for_test(&self, other: &Self) -> bool {
        self.same_allocation(other)
    }
}
