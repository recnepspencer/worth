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
