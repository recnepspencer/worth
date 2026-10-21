//! Frozen thaw granted only by a native consumed-upstream edge and registry custody.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::HeldRequiredSuccessor;
use crate::domain_computation::primary_graph::application_output_demand::PendingUpstream;
use crate::domain_computation::primary_graph::invariant_projection::SelectedPendingConsumedOutput;
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
    ) -> Result<Option<Self>, WorthQueryOutputDemandDenial> {
        Ok(registry
            .take_required_published_successor(selected)?
            .map(|successor| Self {
                held: successor.into_held(),
                schema: std::marker::PhantomData,
            }))
    }

    pub(in super::super) fn from_pending(
        pending: &SelectedPendingConsumedOutput<'_>,
        selected: &PositionedRelationalSnapshot,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<RequiredDelivery<Schema>, WorthQueryOutputDemandDenial> {
        let key = match registry.pending_exact_ready_readmission(pending, selected)? {
            PendingUpstream::Ready(ready) => return Ok(RequiredDelivery::Ready(ready)),
            PendingUpstream::Held(key) => key,
            PendingUpstream::Unavailable(_) => return Ok(RequiredDelivery::Unavailable),
        };
        let Some(held) = registry.take_published_held_successor(&key)? else {
            return Ok(RequiredDelivery::Unavailable);
        };
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

        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<Option<DeliveredAwaited>, WorthQueryOutputDemandDenial> {
        match held::resume_registry_successor(
            phase, runtime, principal, request, branch, custody, self.held, performed,
        )? {
            HeldUpstream::Ready(ready) => Ok(Some(DeliveredAwaited::from_delivery(ready))),
            HeldUpstream::Unfinished | HeldUpstream::GaveBack => Ok(None),
        }
    }
}
