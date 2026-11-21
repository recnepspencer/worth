//! Shared-selection constructor and origin retention on `WorthQueryApplicationSnapshotLease`.

use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::product_operation::{
    SharedSelectedProductOperation, WorthQuerySelectedProductOperation,
};

pub(in crate::domain_computation) trait SnapshotLeaseRuntimeExt: Sized {
    fn from_shared_selected<Schema: worth_query_installation::facade::ApplicationSchema>(
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<Self, CompanionPreflightStop>;

    fn retain_selection_origin_from<Schema: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> bool;
}

impl SnapshotLeaseRuntimeExt for WorthQueryApplicationSnapshotLease {
    fn from_shared_selected<Schema: worth_query_installation::facade::ApplicationSchema>(
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<Self, CompanionPreflightStop> {
        let selected = shared.selected();
        let graph = selected
            .application()
            .runtime
            .primary_graph()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let basis = selected.application_basis().retain_shared()?;
        Ok(Self::from_shared_basis(
            Box::new(basis),
            selected.product().retained_clone(),
            selected.selection_origin(),
            graph.retain_layout(),
        ))
    }

    fn retain_selection_origin_from<Schema: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> bool {
        self.set_selection_origin(selected.product(), selected.selection_origin())
    }
}
