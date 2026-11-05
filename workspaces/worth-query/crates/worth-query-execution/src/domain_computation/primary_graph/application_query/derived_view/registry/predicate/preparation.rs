//! Caller custody binds each genuine Native round; only the accepted child promotes.
mod retirement;
mod round;
use super::{PredicateWorld, SourcePredicateAxis};
use crate::domain_computation::primary_graph::{
    application_query::managed_index::{
        predicate::WatchSlot, SourceImageView, WorthQueryManagedIndexSourceChange,
        WorthQueryManagedIndexWorkDenial,
    },
    live_delivery::WorthQueryLivePredicatePreparation,
    source_facts::SourceFactsAllocation,
};
pub(in crate::domain_computation::primary_graph) use round::{
    PredicateNoticeDisposition, PreparedPredicateNotices,
};
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    runtime::{
        ManagedIndexSourceContextFactory, ManagedIndexSourceContextReceipt,
        ManagedIndexSourceContextShare, ManagedIndexSourcePreparationDenial as Denial,
        ManagedIndexSourcePreparationMetadata, NativeSourceInterpretationIdentity,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};

struct Descriptor {
    world: PredicateWorld,
    before_commit: worth_runtime_world::facade::CompositeCommitIdentity,
    basis: AdmittedRelationalBranchBasis,
    _allocation: Arc<SourceFactsAllocation>,
}
enum Child {
    Empty,
    Bound(ManagedIndexSourceContextShare<round::Round>),
    // Returned opaque Native round closes only after the caller leaves validation.
    Refused { round_custody: round::Round },
}
struct RoundLink {
    child: Child,
    next: Option<Box<RoundLink>>,
    allocation: Arc<SourceFactsAllocation>,
}
pub(in crate::domain_computation::primary_graph) struct PredicatePreparationFactory {
    descriptor: Arc<Descriptor>,
    children: Mutex<Option<Box<RoundLink>>>,
    fence: Option<WorthQueryLivePredicatePreparation>,
}
pub(in crate::domain_computation::primary_graph) struct PredicatePreparationOwner {
    round: ManagedIndexSourceContextShare<round::Round>,
    notices: Arc<PreparedPredicateNotices>,
    _fence: WorthQueryLivePredicatePreparation,
}
pub(in crate::domain_computation::primary_graph) enum PreparedPredicatePublication {
    Matched(Arc<PreparedPredicateNotices>),
    Untracked,
}
impl PredicatePreparationOwner {
    pub(in crate::domain_computation::primary_graph) fn publication_for(
        &self,
        before: &worth_runtime_world::facade::ProductBranchObservation,
        publication: &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) -> PreparedPredicatePublication {
        let descriptor = &self.round.value().descriptor;
        if publication.follows_observation(before)
            && descriptor.world == PredicateWorld::of(before)
            && descriptor.before_commit == *before.selected_commit()
            && descriptor.basis == *before.basis().relational_basis()
        {
            PreparedPredicatePublication::Matched(self.notices.clone())
        } else {
            PreparedPredicatePublication::Untracked
        }
    }
}
/// Created outside Native/registry gates, filled only when exact custody retires.
#[must_use]
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct PredicatePreparationRetirement {
    owner: Option<PredicatePreparationOwner>,
}
impl PredicatePreparationRetirement {
    pub(in crate::domain_computation::primary_graph) fn retire(
        &mut self,
        owner: &mut Option<PredicatePreparationOwner>,
    ) {
        if owner.is_some() {
            assert!(
                self.owner.is_none(),
                "one operation retires one actual predicate preparation"
            );
            self.owner = owner.take();
        }
    }
}
impl PredicatePreparationFactory {
    pub(in crate::domain_computation::primary_graph) fn new(
        execution: &ExecutionRequest<'_, '_>,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        fence: WorthQueryLivePredicatePreparation,
    ) -> Result<Self, Cause> {
        let allocation = SourceFactsAllocation::open(execution)?;
        allocation.claim_arc::<Descriptor>(execution)?;
        let basis = observation.basis().relational_basis();
        Ok(Self {
            descriptor: Arc::new(Descriptor {
                world: PredicateWorld::of(observation),
                before_commit: observation.selected_commit().clone(),
                // Clone shares the SAME admitted owner/root/admission custody;
                // it allocates no descriptor strings or replacement admission.
                basis: basis.clone(),
                _allocation: allocation,
            }),
            children: Mutex::new(None),
            fence: Some(fence),
        })
    }
    /// Called only after Native validation has released its runtime guard.
    pub(in crate::domain_computation::primary_graph) fn promote(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
        accepted: ManagedIndexSourcePreparationMetadata<'_>,
    ) -> Result<PredicatePreparationOwner, Denial> {
        let receipt = accepted.payload().upgrade().ok_or(Denial::ForeignSource)?;
        let received = receipt
            .downcast_ref::<round::Round>()
            .ok_or(Denial::ForeignSource)?;
        if &received.identity != accepted.interpretation()
            || !Arc::ptr_eq(&received.descriptor, &self.descriptor)
        {
            return Err(Denial::ForeignSource);
        }
        let mut children = self
            .children
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let mut promoted = None;
        while let Some(mut link) = children {
            if let Err(cause) = SourceFactsAllocation::work(execution, 1) {
                *self
                    .children
                    .get_mut()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(link);
                return Err(storage(cause));
            }
            children = link.next.take();
            let funding = link.allocation.clone();
            if let Child::Bound(child) = &link.child
                && child.value().identity == *accepted.interpretation()
            {
                if promoted.is_some() {
                    link.next = children;
                    *self
                        .children
                        .get_mut()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(link);
                    return Err(Denial::ForeignSource);
                }
                promoted = Some(child.clone());
            }
            drop(link);
            drop(funding);
        }
        let round = promoted.ok_or(Denial::ForeignSource)?;
        let notices = round.value().freeze(execution)?;
        Ok(PredicatePreparationOwner {
            round,
            notices,
            _fence: self.fence.take().expect("unique caller preparation fence"),
        })
    }
}
impl ManagedIndexSourceContextFactory for PredicatePreparationFactory {
    fn bind(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        interpretation: NativeSourceInterpretationIdentity,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<ManagedIndexSourceContextReceipt, Denial> {
        if basis != &self.descriptor.basis {
            return Err(Denial::ForeignSource);
        }
        // Allocate the refusal escrow before constructing opaque round custody.
        let allocation = SourceFactsAllocation::open(execution).map_err(storage)?;
        allocation
            .claim(execution, size_of::<RoundLink>() as u64)
            .map_err(storage)?;
        let round = round::Round::new(execution, interpretation, self.descriptor.clone())?;
        let mut link = Box::new(RoundLink {
            child: Child::Empty,
            next: None,
            allocation,
        });
        let outcome = match ManagedIndexSourceContextShare::new(execution, round) {
            Ok(child) => {
                let receipt = child.receipt();
                link.child = Child::Bound(child);
                Ok(receipt)
            }
            Err((cause, refused)) => {
                // Never drop the returned opaque value under Native validation.
                link.child = Child::Refused {
                    round_custody: refused,
                };
                Err(storage(cause))
            }
        };
        let mut children = self
            .children
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        link.next = children.take();
        *children = Some(link);
        outcome
    }
}
pub(in crate::domain_computation::primary_graph::application_query) fn route_prepared_predicates(
    execution: &ExecutionRequest<'_, '_>,
    metadata: Option<ManagedIndexSourcePreparationMetadata<'_>>,
    axis: &SourcePredicateAxis,
    scope: worth_relational::facade::identity::EntityId,
    before: SourceImageView<'_>,
    after: SourceImageView<'_>,
    changes: &[WorthQueryManagedIndexSourceChange],
) -> Result<(), Denial> {
    // Native-only preparation has no World publication context and cannot wake.
    let Some(metadata) = metadata else {
        return Ok(());
    };
    let child = metadata.payload().upgrade().ok_or(Denial::ForeignSource)?;
    let round = child
        .downcast_ref::<round::Round>()
        .ok_or(Denial::ForeignSource)?;
    if &round.identity != metadata.interpretation() {
        return Err(Denial::ForeignSource);
    }
    axis.route(execution, &round.descriptor.world, scope, before, after, changes,
        &mut |slot, control| round.record(execution, control, slot, PredicateNoticeDisposition::Refresh))
        .map_err(|cause: WorthQueryManagedIndexWorkDenial| {
            crate::domain_computation::primary_graph::application_query::managed_index::source_preparation_denial(execution, cause)
        })
}
fn storage(cause: Cause) -> Denial {
    Denial::Source(worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause))
}
pub(in crate::domain_computation::primary_graph::application_query) fn mark_prepared_predicates_unavailable(
    execution: &ExecutionRequest<'_, '_>,
    metadata: Option<ManagedIndexSourcePreparationMetadata<'_>>,
    axis: &SourcePredicateAxis,
    scope: worth_relational::facade::identity::EntityId,
    budget: &worth_relational::facade::runtime::CustomInvariantReadBudget<'_>,
) -> Result<(), Denial> {
    let Some(metadata) = metadata else {
        return Ok(());
    };
    let child = metadata.payload().upgrade().ok_or(Denial::ForeignSource)?;
    let round = child
        .downcast_ref::<round::Round>()
        .ok_or(Denial::ForeignSource)?;
    if &round.identity != metadata.interpretation() {
        return Err(Denial::ForeignSource);
    }
    axis.invalidate_scope(execution, &round.descriptor.world, scope, budget,
        &mut |slot, control| round.record(execution, control, slot, PredicateNoticeDisposition::Unavailable))
        .map_err(|cause| crate::domain_computation::primary_graph::application_query::managed_index::source_preparation_denial(execution, cause))
}
