use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::runtime::RelationalRuntime;

use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;
use std::sync::Arc;

use super::WorthQueryGraphWorkSessionIdentity;

/// The only application-query path to the primary graph's Relational runtime.
pub(in crate::domain_computation) struct WorthQueryGraphReadOwnerPort {
    binding: ApplicationSchemaBindingIdentity,
    source: WorthQueryRelationalSourceOwner,
    layout: Arc<WorthQueryPrimaryGraphLayout>,
}

impl WorthQueryGraphReadOwnerPort {
    pub(in crate::domain_computation) fn new(
        binding: ApplicationSchemaBindingIdentity,
        source: WorthQueryRelationalSourceOwner,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
    ) -> Self {
        Self {
            binding,
            source,
            layout,
        }
    }

    pub(super) fn execute_prepared<T>(
        &self,
        binding: &ApplicationSchemaBindingIdentity,
        read: impl FnOnce(&RelationalRuntime, &WorthQueryPrimaryGraphLayout) -> T,
    ) -> Result<T, WorthQueryGraphReadOwnerPortDenial> {
        if &self.binding != binding {
            return Err(WorthQueryGraphReadOwnerPortDenial::ForeignGraph);
        }
        // Release graph custody before resuming a worker panic, so the read-only
        // operation cannot poison the runtime for another root or request.
        Ok(self
            .source
            .with_runtime_unwind_isolated(|runtime| read(runtime, &self.layout)))
    }

    pub(super) fn execute<T>(
        &self,
        binding: &ApplicationSchemaBindingIdentity,
        read: impl FnOnce(&mut RelationalRuntime, &WorthQueryPrimaryGraphLayout) -> T,
    ) -> Result<T, WorthQueryGraphReadOwnerPortDenial> {
        if &self.binding != binding {
            return Err(WorthQueryGraphReadOwnerPortDenial::ForeignGraph);
        }
        Ok(self
            .source
            .with_runtime_mut(|runtime| read(runtime, &self.layout)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryGraphReadOwnerPortDenial {
    ForeignGraph,
}

/// Move-only proof that this exact session performed a read at its retained basis.
pub(in crate::domain_computation) struct WorthQuerySessionGraphReadProof {
    pub(super) session: WorthQueryGraphWorkSessionIdentity,
    pub(super) plan: WorthQueryGraphWorkPlanIdentity,
    pub(super) basis: crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
}

impl WorthQuerySessionGraphReadProof {
    pub(super) fn new(
        session: WorthQueryGraphWorkSessionIdentity,
        plan: WorthQueryGraphWorkPlanIdentity,
        basis: crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
    ) -> Self {
        Self {
            session,
            plan,
            basis,
        }
    }
}
