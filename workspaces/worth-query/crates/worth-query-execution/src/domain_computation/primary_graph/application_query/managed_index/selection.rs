//! The local original-request proof lends only sealed immutable rows to Native.
use super::{
    invocation::controlled,
    retention::{ManagedIndexState, ReadyIndex},
    WorthQueryManagedIndexStop as Stop, WorthQueryManagedIndexWorkDenial as WorkDenial,
};
use crate::domain_computation::primary_graph::access_strategy::coverage::{
    ReceivedSelection, RetainedCoverage,
};
use crate::domain_computation::{
    primary_graph::{
        application_query::{
            observed_source::ManagedSourceDependencies,
            one_shot::outcome::material::MaterialCompletion,
        },
        source_facts::SourceFactsAllocation,
    },
    provider_session::{
        WorthQueryCompletedSourceRead, WorthQueryGraphWorkSessionIdentity,
        WorthQueryManagedGraphWorkSession, WorthQueryManagedIndexSelectionProof,
        WorthQueryManagedIndexSessionDenial,
    },
};
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionRequestArrayBuilder};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::{
    identity::EntityId, transactions::RelationalExecutionDenialCause as Cause,
};
mod maintained;

#[derive(Debug)]
pub(super) enum SelectionDenial {
    Session(WorthQueryManagedIndexSessionDenial),
    Work(WorkDenial),
    Execution(Cause),
    ForeignSelection,
}
impl From<Cause> for SelectionDenial {
    fn from(cause: Cause) -> Self {
        Self::Execution(cause)
    }
}

trait RootCurrency: Send + Sync {
    fn maintained_observation(
        &self,
    ) -> Option<Arc<worth_relational::facade::runtime::ManagedIndexSourceObservation>> {
        None
    }
    fn current(
        &self,
        association: &WorthQueryInstalledManagedIndexParent,
        coverage: Option<&RetainedCoverage>,
    ) -> bool;
}
struct RootEpoch<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    state: Arc<ManagedIndexState<S, I>>,
    root: Arc<ReadyIndex<S, I>>,
    epoch: u64,
    _allocation: Arc<SourceFactsAllocation>,
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> RootCurrency for RootEpoch<S, I> {
    fn current(
        &self,
        association: &WorthQueryInstalledManagedIndexParent,
        coverage: Option<&RetainedCoverage>,
    ) -> bool {
        self.state.association.same_association(association)
            && self.state.holds_ready(self.epoch, &self.root)
            && coverage.is_none_or(|c| c.admits(self.state.adapter.coverage_authority()))
    }
}

/// No public constructor and no SDK/session loan on the worker-facing rows.
pub(in crate::domain_computation::primary_graph::application_query) struct SelectionRows {
    parent: WorthQueryGraphWorkSessionIdentity,
    association: WorthQueryInstalledManagedIndexParent,
    scope: EntityId,
    roots: ExecutionArray<EntityId>,
    dependencies: Arc<ManagedSourceDependencies>,
    currency: Arc<dyn RootCurrency>,
    pub(in crate::domain_computation::primary_graph::application_query) coverage:
        Option<Arc<RetainedCoverage>>,
}
impl SelectionRows {
    pub(in crate::domain_computation::primary_graph::application_query) fn admits(
        &self,
        parent: WorthQueryGraphWorkSessionIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        scope: EntityId,
    ) -> bool {
        self.parent == parent
            && self.scope == scope
            && self.association.binding_identity() == binding
            && self.currency.current(&self.association, self.coverage.as_deref())
            && (self.association.definition().parts().membership == worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership::Candidates || self.coverage.is_some())
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn roots(
        &self,
    ) -> &[EntityId] {
        self.roots.elements()
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn association(
        &self,
    ) -> &WorthQueryInstalledManagedIndexParent {
        &self.association
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn dependencies(
        &self,
    ) -> &Arc<ManagedSourceDependencies> {
        &self.dependencies
    }
}

pub(super) struct ManagedSelectionLease<'parent, 'request, 'authority> {
    proof: WorthQueryManagedIndexSelectionProof<'parent, 'request, 'authority>,
    source: Arc<WorthQueryCompletedSourceRead>,
    rows: SelectionRows,
}
/// Owned evidence is derived only while the real maintained selection loan lives.
/// It describes opening coverage; fresh requests still admit every later READ.
pub(in crate::domain_computation::primary_graph::application_query) struct MaintainedWatchSelection
{
    pub(in crate::domain_computation::primary_graph::application_query) association:
        WorthQueryInstalledManagedIndexParent,
    pub(in crate::domain_computation::primary_graph::application_query) scope: EntityId,
    pub(in crate::domain_computation::primary_graph::application_query) native:
        Arc<worth_relational::facade::runtime::ManagedIndexSourceObservation>,
    _source: Arc<WorthQueryCompletedSourceRead>,
    _coverage: Arc<RetainedCoverage>,
    _dependencies: Arc<ManagedSourceDependencies>,
}
impl<'parent, 'request, 'authority> ManagedSelectionLease<'parent, 'request, 'authority> {
    pub(super) fn capture_watch(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
    ) -> Result<MaintainedWatchSelection, SelectionDenial> {
        let rows = self.rows(parent, request)?;
        Ok(MaintainedWatchSelection {
            association: rows.association.clone(),
            scope: rows.scope,
            native: rows
                .currency
                .maintained_observation()
                .ok_or(SelectionDenial::ForeignSelection)?,
            _source: self.source.clone(),
            _coverage: rows
                .coverage
                .clone()
                .ok_or(SelectionDenial::ForeignSelection)?,
            _dependencies: rows.dependencies.clone(),
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn issue<S: ApplicationSchema + 'static, I: ApplicationManagedIndexBinding<S>>(
        proof: WorthQueryManagedIndexSelectionProof<'parent, 'request, 'authority>,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
        state: Arc<ManagedIndexState<S, I>>,
        root: Arc<ReadyIndex<S, I>>,
        epoch: u64,
        selected: ReceivedSelection,
    ) -> Result<Self, SelectionDenial> {
        proof.consult().map_err(SelectionDenial::Session)?;
        if !proof.admits(parent, request, &state.association, &root.source.terminal)
            || !state.holds_ready(epoch, &root)
        {
            return Err(SelectionDenial::ForeignSelection);
        }
        let execution = proof.execution();
        let (slots, coverage) = selected.into_parts();
        let roots = controlled(&execution, |control| {
            let mut roots = ExecutionRequestArrayBuilder::allocate(execution, slots.len())
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            for slot in slots.elements() {
                control.checkpoint(1)?;
                if slot.owner != root.source.terminal.session_identity() {
                    return Err(control.refuse(Stop::ForeignSourceSlot).into());
                }
                let original = root
                    .source
                    .material
                    .order
                    .get(slot.ordinal)
                    .ok_or_else(|| control.refuse(Stop::SourceSlotUnavailable))?;
                let row = root
                    .source
                    .material
                    .tree
                    .at(*original)
                    .ok_or_else(|| control.refuse(Stop::SourceSlotUnavailable))?;
                roots
                    .push(row.entity_id())
                    .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            }
            roots
                .seal()
                .map_err(|cause| control.refuse(Stop::Storage(cause)).into())
        })
        .map_err(SelectionDenial::Work)?;
        let allocation = SourceFactsAllocation::open(&execution)?;
        MaterialCompletion::new(&execution, &allocation).arc::<RootEpoch<S, I>>()?;
        let source = Arc::clone(&root.source.terminal);
        let dependencies = Arc::clone(&root.source.dependencies);
        let association = state.association.clone();
        let scope = source.scope();
        let currency: Arc<dyn RootCurrency> = Arc::new(RootEpoch {
            state,
            root,
            epoch,
            _allocation: allocation,
        });
        let lease = Self {
            proof,
            source,
            rows: SelectionRows {
                parent: parent.identity(),
                association,
                scope,
                roots,
                dependencies,
                currency,
                coverage,
            },
        };
        lease.rows(parent, request)?;
        Ok(lease)
    }

    pub(super) fn rows(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
    ) -> Result<&SelectionRows, SelectionDenial> {
        self.proof.consult().map_err(SelectionDenial::Session)?;
        if !self
            .proof
            .admits(parent, request, &self.rows.association, &self.source)
            || !self
                .rows
                .currency
                .current(&self.rows.association, self.rows.coverage.as_deref())
        {
            return Err(SelectionDenial::ForeignSelection);
        }
        Ok(&self.rows)
    }
}
