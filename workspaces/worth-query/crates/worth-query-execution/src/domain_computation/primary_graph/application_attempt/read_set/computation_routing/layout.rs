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

#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) struct CheckedReadLayout
{
    pub(super) contracts: Arc<WorthQueryCompiledApplicationOperationContracts>,
    pub(super) scopes:
        crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
    pub(super) handlers: BTreeMap<WorthQueryApplicationFactKey, usize>,
    pub(super) root: EntityId,
    pub(super) decision_count: usize,
    pub(super) calls: BTreeSet<ComputationRead>,
    pub(super) computation_ordinals: Arc<[usize]>,
}
impl CheckedReadLayout {
    pub(super) fn new(
        contracts: Arc<WorthQueryCompiledApplicationOperationContracts>,
        scopes: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        handlers: BTreeMap<WorthQueryApplicationFactKey, usize>,
        root: EntityId,
        decision_count: usize,
        calls: BTreeSet<ComputationRead>,
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
            contracts,
            scopes,
            handlers,
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
    pub(super) fn charged_bytes(&self) -> Option<u64> {
        match self {
            Self::Checked(layout) => {
                use worth_execution::ChargedBytes;
                let base = (size_of::<CheckedReadLayout>() + 2 * size_of::<usize>()) as u64;
                layout.handlers.keys().try_fold(
                    base.checked_add(layout.scopes.backing_bytes()?)?
                        .checked_add(
                            (layout.calls.len().checked_mul(
                                11 * size_of::<ComputationRead>() + 16 * size_of::<usize>(),
                            )?) as u64,
                        )?
                        .checked_add(
                            (layout
                                .computation_ordinals
                                .len()
                                .checked_mul(size_of::<usize>())?)
                                as u64,
                        )?
                        .checked_add(2 * size_of::<usize>() as u64)?,
                    |sum, key| {
                        super::super::super::super::fact_verification_observation::completion_metadata();
                        sum.checked_add(
                            (11 * size_of::<(WorthQueryApplicationFactKey, usize)>()
                                + 16 * size_of::<usize>()) as u64,
                        )?
                        .checked_add(key.additional_charged_bytes())
                    },
                )
            }
            #[cfg(test)]
            Self::OwnerFixture => Some(0),
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
        if !shape
            .calls()
            .inspect(|_| {
                super::super::super::super::fact_verification_observation::completion_call_roster()
            })
            .eq(layout.calls.iter().copied())
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
