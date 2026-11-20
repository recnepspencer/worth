//! Primary-graph typed adapters for `WorthQueryManagedGraphWorkSession`.

use worth_query_execution_runtime::facade::provider_session::WorthQueryManagedGraphWorkSession;

use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::{
    StorageControl, StoreDenial,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

pub(crate) trait ManagedGraphWorkSessionPrimaryExt {
    fn arm_coupled_handler(&mut self);

    fn coupled_handler_typed(&self) -> Option<&ProducerHandlerPreparation>;

    fn coupled_handler_typed_mut(&mut self) -> Option<&mut ProducerHandlerPreparation>;

    fn take_coupled_handler_typed(&mut self) -> Option<ProducerHandlerPreparation>;

    fn mutation_handle(&self) -> Option<&WorthQueryPrimaryGraphIntegrationHandle>;

    fn mutation_lease_typed(&self) -> Option<&WorthQueryApplicationSnapshotLease>;

    fn mutation_lease_typed_mut(&mut self) -> Option<&mut WorthQueryApplicationSnapshotLease>;

    fn take_mutation_lease_typed(&mut self) -> Option<WorthQueryApplicationSnapshotLease>;

    fn share_mutation_lease(
        &mut self,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<WorthQueryApplicationSnapshotLease>, StoreDenial>;
}

impl ManagedGraphWorkSessionPrimaryExt for WorthQueryManagedGraphWorkSession {
    fn arm_coupled_handler(&mut self) {
        self.arm_coupled_handler_any(Box::new(ProducerHandlerPreparation::new()));
    }

    fn coupled_handler_typed(&self) -> Option<&ProducerHandlerPreparation> {
        self.coupled_handler()
            .map(|any| {
                any.downcast_ref::<ProducerHandlerPreparation>()
                    .expect("coupled handler is ProducerHandlerPreparation")
            })
    }

    fn coupled_handler_typed_mut(&mut self) -> Option<&mut ProducerHandlerPreparation> {
        self.coupled_handler_mut().map(|any| {
            any.downcast_mut::<ProducerHandlerPreparation>()
                .expect("coupled handler is ProducerHandlerPreparation")
        })
    }

    fn take_coupled_handler_typed(&mut self) -> Option<ProducerHandlerPreparation> {
        self.take_coupled_handler().map(|any| {
            *any.downcast::<ProducerHandlerPreparation>()
                .expect("coupled handler is ProducerHandlerPreparation")
        })
    }

    fn mutation_handle(&self) -> Option<&WorthQueryPrimaryGraphIntegrationHandle> {
        self.mutation_lease_typed()
            .map(WorthQueryApplicationSnapshotLease::handle)
    }

    fn mutation_lease_typed(&self) -> Option<&WorthQueryApplicationSnapshotLease> {
        self.mutation_lease().map(|lease| {
            lease
                .as_any()
                .downcast_ref::<WorthQueryApplicationSnapshotLease>()
                .expect("primary mutation lease is WorthQueryApplicationSnapshotLease")
        })
    }

    fn mutation_lease_typed_mut(&mut self) -> Option<&mut WorthQueryApplicationSnapshotLease> {
        self.mutation_lease_mut().map(|lease| {
            lease
                .as_any_mut()
                .downcast_mut::<WorthQueryApplicationSnapshotLease>()
                .expect("primary mutation lease is WorthQueryApplicationSnapshotLease")
        })
    }

    fn take_mutation_lease_typed(&mut self) -> Option<WorthQueryApplicationSnapshotLease> {
        self.take_mutation_lease().map(|lease| {
            *lease
                .into_any()
                .downcast::<WorthQueryApplicationSnapshotLease>()
                .expect("primary mutation lease is WorthQueryApplicationSnapshotLease")
        })
    }

    fn share_mutation_lease(
        &mut self,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<WorthQueryApplicationSnapshotLease>, StoreDenial> {
        let Some(lease) = self.take_mutation_lease_typed() else {
            return Ok(None);
        };
        let (original, retained) = lease.share_original(control)?;
        self.restore_mutation_lease(Box::new(original));
        Ok(Some(retained))
    }
}
