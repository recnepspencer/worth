//! Exact authority retained while an accepted demand is refreshed.

use std::sync::Arc;

use super::{
    DemandRegistryState, DemandState, WorthQueryAcceptedOutputAuthority,
    WorthQueryOutputCheckpoint, WorthQueryOutputDemandInterest, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::{
    output_lineage::PublishedStableLineage, WorthQueryApplicationCommitReceipt,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// The accepted source being replaced. A Stable alias is identified by its
/// own published cell and Product observation, never by its performed origin.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum OutputRefreshPredecessor<'a> {
    Committed(&'a WorthQueryApplicationCommitReceipt),
    Stable {
        interest: &'a WorthQueryOutputDemandInterest,
        published: &'a PublishedStableLineage,
    },
    /// A restored output that is no longer current executes Fresh.
    Restored(&'a super::WorthQueryRestoredAcceptedOutput),
}

impl<'a> OutputRefreshPredecessor<'a> {
    pub(in crate::domain_computation::primary_graph) fn of(
        authority: &'a WorthQueryAcceptedOutputAuthority,
        interest: &'a WorthQueryOutputDemandInterest,
    ) -> Self {
        match authority {
            WorthQueryAcceptedOutputAuthority::Committed(receipt) => Self::Committed(receipt),
            WorthQueryAcceptedOutputAuthority::Stable(published) => Self::Stable {
                interest,
                published,
            },
            WorthQueryAcceptedOutputAuthority::Restored(restored) => Self::Restored(restored),
        }
    }
}

impl OutputRefreshPredecessor<'_> {
    pub(super) fn key_identity(self) -> [u8; 32] {
        match self {
            Self::Committed(receipt) => *receipt.idempotency_binding().key_identity(),
            Self::Stable { published, .. } => published.idempotency_key_identity(),
            Self::Restored(restored) => restored.checkpoint.idempotency_key,
        }
    }

    pub(super) fn matches_ready(self, authority: &WorthQueryAcceptedOutputAuthority) -> bool {
        match (self, authority) {
            (Self::Committed(expected), WorthQueryAcceptedOutputAuthority::Committed(actual)) => {
                actual.is_same_authoritative_commit(expected)
            }
            (Self::Stable { published, .. }, WorthQueryAcceptedOutputAuthority::Stable(actual)) => {
                actual.same_publication(published)
            }
            (Self::Restored(expected), WorthQueryAcceptedOutputAuthority::Restored(actual)) => {
                actual.checkpoint.idempotency_key == expected.checkpoint.idempotency_key
                    && Arc::ptr_eq(&actual.correspondence, &expected.correspondence)
            }
            _ => false,
        }
    }

    pub(super) fn validate_stable_interest(
        self,
        owner: &WorthQueryOutputDemandRegistry,
        state: &DemandRegistryState,
        replacement: &WorthQueryOutputDemandKey,
        source_scope: WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Self::Stable { interest, .. } = self else {
            return Ok(());
        };
        if !Arc::ptr_eq(&interest.owner.state, &owner.state)
            || interest
                .key
                .replacement_order(replacement)
                .is_none_or(|order| order.is_gt())
        {
            return Err(stale_predecessor());
        }
        let Some(record) = state.records.get(&interest.key) else {
            return Err(stale_predecessor());
        };
        if record.product_occurrence != occurrence || record.source_scope != Some(source_scope) {
            return Err(stale_predecessor());
        }
        let DemandState::Output(output) = &record.state else {
            return Err(stale_predecessor());
        };
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint.as_ref() else {
            return Err(stale_predecessor());
        };
        if !self.matches_ready(&completion.authority) {
            return Err(stale_predecessor());
        }
        Ok(())
    }
}

fn stale_predecessor() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "the accepted stable predecessor no longer owns this demand",
    )
}
