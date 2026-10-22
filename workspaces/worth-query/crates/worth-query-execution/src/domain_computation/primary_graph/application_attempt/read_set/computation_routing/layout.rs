//! Installed scope/handler correspondence of an actually completed cold read.
use super::*;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts;
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SealedReadLayout {
    Checked(Arc<CheckedReadLayout>),
    #[cfg(test)]
    OwnerFixture,
}

pub(in crate::domain_computation::primary_graph::application_attempt::read_set) struct CheckedReadLayout
{
    pub(super) contracts: Arc<WorthQueryCompiledApplicationOperationContracts>,
    pub(super) scopes:
        crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
    pub(super) handlers: Arc<BTreeMap<WorthQueryApplicationFactKey, usize>>,
    pub(super) root: EntityId,
    pub(super) decision_count: usize,
    pub(super) calls: worth_execution::ExecutionOrderedMap<ComputationRead, ()>,
    bound_shape:
        Option<Arc<crate::domain_computation::primary_graph::invariant_projection::CallShape>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub(super) computation_ordinals: Arc<[usize]>,
}
impl CheckedReadLayout {
    pub(super) fn new(
        contracts: Arc<WorthQueryCompiledApplicationOperationContracts>,
        scopes: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        handlers: BTreeMap<WorthQueryApplicationFactKey, usize>,
        root: EntityId,
        decision_count: usize,
        calls: worth_execution::ExecutionOrderedMap<ComputationRead, ()>,
        computation_ordinals: Arc<[usize]>,
    ) -> Self {
        assert!(
            scopes.same_contract(&contracts),
            "completed scope positions belong to the actual installed contract"
        );
        assert_eq!(
            scopes.len(),
            decision_count,
            "completed scopes cover every decision ordinal"
        );
        Self {
            bound_shape: None,
            _memory: None,
            contracts,
            scopes,
            handlers: Arc::new(handlers),
            root,
            decision_count,
            calls,
            computation_ordinals,
        }
    }
}

impl SealedReadLayout {
    pub(super) fn checked(&self) -> Option<&Arc<CheckedReadLayout>> {
        match self {
            Self::Checked(layout) => Some(layout),
            #[cfg(test)]
            Self::OwnerFixture => None,
        }
    }
}

/// Exact correspondence established at a real completed read. Shape identity
/// alone has no source/snapshot authority; this also holds its checked layout.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct CheckedCallShape {
    pub(super) shape:
        Arc<crate::domain_computation::primary_graph::invariant_projection::CallShape>,
    layout: Arc<CheckedReadLayout>,
}
impl CheckedCallShape {
    pub(in crate::domain_computation::primary_graph) fn same_shape(
        &self,
        shape: &Arc<crate::domain_computation::primary_graph::invariant_projection::CallShape>,
    ) -> bool {
        Arc::ptr_eq(&self.shape, shape)
    }
}
impl SealedComputationFacts {
    pub(in crate::domain_computation::primary_graph) fn bind_call_shape(
        &self,
        shape: &Arc<crate::domain_computation::primary_graph::invariant_projection::CallShape>,
        previous: Option<&CheckedCallShape>,
    ) -> Option<CheckedCallShape> {
        let layout = self.layout.checked()?;
        if let Some(previous) = previous
            && Arc::ptr_eq(&previous.shape, shape)
            && Arc::ptr_eq(&previous.layout, layout)
        {
            return Some(previous.clone());
        }
        if layout
            .bound_shape
            .as_ref()
            .is_some_and(|bound| Arc::ptr_eq(bound, shape))
        {
            return Some(CheckedCallShape {
                shape: Arc::clone(shape),
                layout: Arc::clone(layout),
            });
        }
        if !shape
            .calls()
            .inspect(|_| {
                super::super::super::super::fact_verification_observation::completion_call_roster()
            })
            .eq(layout.calls.keys().copied())
        {
            return None;
        }
        Some(CheckedCallShape {
            shape: Arc::clone(shape),
            layout: Arc::clone(layout),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn has_call_shape(
        &self,
        binding: &CheckedCallShape,
    ) -> bool {
        self.layout
            .checked()
            .is_some_and(|layout| Arc::ptr_eq(layout, &binding.layout))
    }
}

impl PartialEq for CheckedReadLayout {
    fn eq(&self, other: &Self) -> bool {
        self.contracts == other.contracts
            && self.scopes == other.scopes
            && self.handlers == other.handlers
            && self.root == other.root
            && self.decision_count == other.decision_count
            && self.calls == other.calls
            && self.computation_ordinals == other.computation_ordinals
    }
}
impl Eq for CheckedReadLayout {}
impl CheckedReadLayout {
    // Only after successful native closure and exact total-key reattribution.
    pub(super) fn after_transition(
        &self,
        transition: &crate::domain_computation::primary_graph::invariant_projection::CallShapeTransition,
        control: super::super::super::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<Arc<Self>, super::super::super::retained_decision_facts::StoreDenial> {
        use super::reattribution::storage_denial;
        control.check_live()?;
        let memory = if control.policy().uses_accounting() {
            control
                .policy()
                .reserve_memory((size_of::<Self>() + 2 * size_of::<usize>()) as u64)
        } else {
            Ok(None)
        };
        control.check_live()?;
        let memory = memory?;
        let mut calls = self.calls.clone();
        for read in transition.retired() {
            assert!(
                calls
                    .remove_in(&read, control.policy())
                    .map_err(storage_denial)?,
                "native prior contains retired call"
            );
        }
        for read in transition.inserted() {
            assert!(
                !calls.contains_key(&read),
                "native prior excludes inserted call"
            );
            calls
                .insert_cloned_in(&read, &(), control.policy())
                .map_err(storage_denial)?;
        }
        Ok(Arc::new(Self {
            contracts: Arc::clone(&self.contracts),
            scopes: self.scopes.clone(),
            handlers: Arc::clone(&self.handlers),
            root: self.root,
            decision_count: self.decision_count,
            calls,
            bound_shape: Some(Arc::clone(transition.target())),
            computation_ordinals: Arc::clone(&self.computation_ordinals),
            _memory: memory,
        }))
    }
}

impl std::fmt::Debug for CheckedReadLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckedReadLayout")
            .field("root", &self.root)
            .field("decision_count", &self.decision_count)
            .field("call_count", &self.calls.len())
            .finish_non_exhaustive()
    }
}
