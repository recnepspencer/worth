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

    pub(super) fn pin_prepared(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &ApplicationSchemaBindingIdentity,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        select_indexes: impl FnOnce(
            &WorthQueryPrimaryGraphLayout,
        ) -> Vec<worth_relational::facade::indexes::DerivedIndexId>,
    ) -> Result<
        (
            worth_relational::facade::runtime::RelationalPinnedRead,
            Arc<WorthQueryPrimaryGraphLayout>,
        ),
        super::WorthQueryManagedGraphReadDenial,
    > {
        use super::WorthQueryManagedGraphReadDenial as Denial;
        if &self.binding != binding {
            return Err(Denial::ForeignGraph);
        }
        let selected_indexes = select_indexes(&self.layout);
        let read = self
            .source
            .with_runtime(execution, |runtime| {
                runtime
                    .read_truth()
                    .pin_read_capability(snapshot, &selected_indexes, execution)
            })
            .map_err(|denial| {
                use worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial as Pin;
                match denial {
                    Pin::Request(stop) => Denial::RequestStop(stop),
                    Pin::SnapshotUnavailable | Pin::IndexNotInstalled(_) => Denial::ForeignBasis,
                }
            })?;
        Ok((read, Arc::clone(&self.layout)))
    }

    pub(super) fn execute<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &ApplicationSchemaBindingIdentity,
        read: impl FnOnce(&mut RelationalRuntime, &WorthQueryPrimaryGraphLayout) -> T,
    ) -> Result<T, WorthQueryGraphReadOwnerPortDenial> {
        if &self.binding != binding {
            return Err(WorthQueryGraphReadOwnerPortDenial::ForeignGraph);
        }
        Ok(self
            .source
            .with_runtime_mut(execution, |runtime| read(runtime, &self.layout)))
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
