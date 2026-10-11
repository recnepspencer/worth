//! Frozen thaw granted only by a native consumed-upstream edge and registry custody.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::{HeldRequiredSuccessor, OutputRowStage, PendingUpstream},
    invariant_projection::SelectedPendingConsumedOutput,
};
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

/// The actual moved successor. Neither a row key nor a foreign interest grants it.
pub(in super::super) struct RegistryDelivery<Schema: ApplicationSchema> {
    held: HeldRequiredSuccessor,
    schema: std::marker::PhantomData<Schema>,
}
pub(in super::super) enum RequiredDelivery<Schema: ApplicationSchema> {
    Ready(SelectedReadyReadmission),
    Delivery(RegistryDelivery<Schema>),
    Unavailable,
}
impl<Schema: ApplicationSchema + 'static> RegistryDelivery<Schema> {
    pub(in super::super) fn from_required_claim(
        selected: &SelectedReadyReadmission,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Self>, WorthQueryOutputDemandDenial> {
        Ok(registry
            .take_required_published_successor(selected, admission)?
            .map(|successor| Self {
                held: successor.into_held(),
                schema: std::marker::PhantomData,
            }))
    }

    pub(in super::super) fn from_pending(
        pending: &SelectedPendingConsumedOutput<'_>,
        selected: &PositionedRelationalSnapshot,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RequiredDelivery<Schema>, WorthQueryOutputDemandDenial> {
        let key = match registry.pending_exact_ready_readmission(pending, selected, admission)? {
            PendingUpstream::Ready(ready) => return Ok(RequiredDelivery::Ready(ready)),
            PendingUpstream::Held(key) => key,
            PendingUpstream::Unavailable(_) => return Ok(RequiredDelivery::Unavailable),
        };
        let Some(held) = registry.take_held_successor(&key, admission)? else {
            return Ok(RequiredDelivery::Unavailable);
        };
        let progress = held
            .downcast_ref::<held::HeldFrameSuccessor<Schema>>()
            .expect("the registry holds this runtime's successor");
        let stage = registry.row_stage_admitted(progress.progress.interest(), admission);
        if !matches!(stage, Ok(OutputRowStage::Published)) {
            // A refused lookup cannot lose the actual held successor.
            drop(registry.hold_required_successor(held, held::held_key::<Schema>));
            return stage.map(|_| RequiredDelivery::Unavailable);
        }
        Ok(RequiredDelivery::Delivery(Self {
            held,
            schema: std::marker::PhantomData,
        }))
    }
    pub(in super::super) fn deliver(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        custody: &mut RequiredContinuations<Schema>,
        admission: &mut InvalidationEditAdmission,
        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<Option<DeliveredAwaited>, WorthQueryOutputDemandDenial> {
        match held::resume_registry_successor(
            phase, runtime, principal, request, branch, custody, self.held, admission, performed,
        )? {
            HeldUpstream::Ready(_) => Ok(Some(awaited_delivery::delivered_registry())),
            HeldUpstream::Unfinished | HeldUpstream::GaveBack => Ok(None),
        }
    }
}
