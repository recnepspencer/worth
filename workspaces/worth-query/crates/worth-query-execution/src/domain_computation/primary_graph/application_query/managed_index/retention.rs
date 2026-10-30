//! One registered association's cache status and original-funded root custody.
use super::{adapter::InstalledIndexProvider, source_image::SourceImage};
use crate::domain_computation::{
    execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    primary_graph::{
        application_query::one_shot::outcome::material::MaterialCompletion,
        source_facts::SourceFactsAllocation, WorthQuerySelectedProductOperation,
    },
    provider_session::{WorthQueryManagedGraphWorkSession, WorthQueryManagedIndexSessionDenial},
};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::{identity::EntityId, transactions::RelationalExecutionDenialCause};
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchIncarnation};

mod association;
mod eviction;
pub(in crate::domain_computation::primary_graph::application_query) use association::ManagedIndexRegistrationKey;
mod maintained;
mod transitions;

/// Retained cache status; it does not assert current authorization or Source availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexLifecycle {
    Created,
    Ready,
    Evicted,
    Rebuilding,
    Failed,
    Disposed,
}

/// Why the retained association could not perform a requested lifecycle step.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_query) enum WorthQueryManagedIndexLifecycleDenial
{
    Execution(RelationalExecutionDenialCause),
    ForeignAssociation,
    RegistrationCapacityExceeded,
    RegistrationIdentityExhausted,
    Disposed,
    BuildInProgress,
    EpochExhausted,
    StaleImage,
    NotReady,
    InvariantOwned,
    Session(WorthQueryManagedIndexSessionDenial),
}
impl From<RelationalExecutionDenialCause> for WorthQueryManagedIndexLifecycleDenial {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Execution(cause)
    }
}

pub(in crate::domain_computation::primary_graph::application_query) struct ReadyIndex<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    pub(super) source: Arc<SourceImage<S, I>>,
    pub(super) cache: worth_relational::facade::mvcc::ManagedIndexPublicCache,
    // The header follows every independent root share, including eviction.
    _allocation: Arc<SourceFactsAllocation>,
}

/// Native owns only pure material. Completed Source READ custody remains in ReadyIndex.
pub(super) struct ReadyIndexMaterial {
    pub(super) source: Arc<super::source_image::SourceMaterial>,
    pub(super) nodes: Arc<dyn Any + Send + Sync>,
    _allocation: Arc<SourceFactsAllocation>,
}
struct Retained<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    epoch: u64,
    current: CompositeCommitIdentity,
    status: WorthQueryManagedIndexLifecycle,
    root: Option<RetainedIndexImage<S, I>>,
}
enum RetainedIndexImage<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    Completed(Arc<ReadyIndex<S, I>>),
    // Status owns one observation directly. The funded registration header
    // includes this slot; replacing it creates no accumulating header charge.
    Maintained(worth_relational::facade::runtime::ManagedIndexSourceObservation),
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> RetainedIndexImage<S, I> {
    fn retire(self) {
        if let Self::Completed(root) = &self {
            drop(root.cache.retire());
        }
        drop(self);
    }
}

pub(in crate::domain_computation::primary_graph::application_query) struct ManagedIndexState<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    pub(in crate::domain_computation::primary_graph::application_query) association:
        WorthQueryInstalledManagedIndexParent,
    pub(super) adapter: Arc<dyn InstalledIndexProvider<S, I>>,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    principal: EntityId,
    scope: EntityId,
    incarnation: ProductBranchIncarnation,
    retained: Mutex<Retained<S, I>>,
    pub(in crate::domain_computation::primary_graph::application_query) allocation:
        Arc<SourceFactsAllocation>,
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(super) fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        association: WorthQueryInstalledManagedIndexParent,
        adapter: Arc<dyn InstalledIndexProvider<S, I>>,
        parent: &WorthQueryManagedGraphWorkSession,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
    ) -> Result<Arc<Self>, WorthQueryManagedIndexLifecycleDenial> {
        use WorthQueryManagedIndexLifecycleDenial as Denial;
        let allocation = SourceFactsAllocation::open(execution)?;
        let storage = MaterialCompletion::new(execution, &allocation);
        storage.work(1)?;
        let context = parent
            .retain_query_source_context()
            .map_err(|_| Denial::ForeignAssociation)?;
        if context.runtime() != selected.application().runtime.authority_identity()
            || !adapter
                .installed()
                .is_installed_in(selected.application().installed_schema())
            || !adapter.installed().admits_association(&association)
            || parent.binding() != association.binding_identity()
            || parent.obligation() != association.access_contract().parent_obligation()
        {
            return Err(Denial::ForeignAssociation);
        }
        if !parent.admits_managed_product(
            selected.product().read_lease_ref(),
            &mut |units| storage.work(units),
            || RelationalExecutionDenialCause::WorkCounterOverflow,
        )? {
            return Err(Denial::ForeignAssociation);
        }
        // The unique World incarnation includes its genuine owner and a never
        // reused ordinal; no branch-name heap copy participates in the key.
        storage.arc::<Self>()?;
        Ok(Arc::new(Self {
            association,
            adapter,
            runtime: context.runtime(),
            principal: context.principal(),
            scope: context.scope(),
            incarnation: selected.product().observation().lifecycle_incarnation(),
            retained: Mutex::new(Retained {
                epoch: 0,
                current: selected.product().selected_commit().clone(),
                status: WorthQueryManagedIndexLifecycle::Created,
                root: None,
            }),
            allocation,
        }))
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn same_association(
        &self,
        other: &Self,
    ) -> bool {
        self.runtime == other.runtime
            && self.principal == other.principal
            && self.scope == other.scope
            && self.incarnation == other.incarnation
            && self.association.same_association(&other.association)
            && Arc::ptr_eq(&self.adapter, &other.adapter)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn lifecycle(
        &self,
    ) -> WorthQueryManagedIndexLifecycle {
        let (status, cache) = {
            let retained = self
                .retained
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let cache = match &retained.root {
                Some(RetainedIndexImage::Completed(root)) => Some(root.cache.clone()),
                _ => None,
            };
            (retained.status, cache)
        };
        if status == WorthQueryManagedIndexLifecycle::Ready
            && cache.is_some_and(|cache| !cache.is_at_head())
        {
            WorthQueryManagedIndexLifecycle::Evicted
        } else {
            status
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn is_disposed(
        &self,
    ) -> bool {
        self.retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .status
            == WorthQueryManagedIndexLifecycle::Disposed
    }
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ReadyIndex<S, I> {
    pub(super) fn retain(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Arc<SourceImage<S, I>>,
        cache: worth_relational::facade::mvcc::ManagedIndexPublicCache,
        allocation: Arc<SourceFactsAllocation>,
    ) -> Result<Arc<Self>, WorthQueryManagedIndexLifecycleDenial> {
        execution
            .consult()
            .map_err(|cause| WorthQueryManagedIndexLifecycleDenial::Execution(cause.into()))?;
        Ok(Arc::new(Self {
            source,
            cache,
            _allocation: allocation,
        }))
    }
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    super::super::derived_view::RegisteredManagedIndex for ManagedIndexState<S, I>
{
    fn dispose(&self) {
        self.close();
    }
}

impl ReadyIndexMaterial {
    pub(super) fn retain(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Arc<super::source_image::SourceMaterial>,
        nodes: Arc<dyn Any + Send + Sync>,
    ) -> Result<Arc<Self>, WorthQueryManagedIndexLifecycleDenial> {
        let allocation = SourceFactsAllocation::open(execution)?;
        MaterialCompletion::new(execution, &allocation).arc::<Self>()?;
        Ok(Arc::new(Self {
            source,
            nodes,
            _allocation: allocation,
        }))
    }
}
