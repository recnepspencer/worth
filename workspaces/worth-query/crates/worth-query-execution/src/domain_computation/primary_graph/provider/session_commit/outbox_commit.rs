//! Exact dispatch-outbox identity sealed from the authoritative commit artifact.

use {crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord, crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryPendingDispatchOutbox};
use worth_relational::facade::transactions::{CommitResult, RecordRef};
use crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBinding;
use crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBindingDenial;



/// Owner-sealed outcome of binding one pending outbox create to its commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxResolution {
    binding: Result<
        Option<WorthQueryCommittedDispatchOutboxBinding>,
        WorthQueryCommittedDispatchOutboxBindingDenial,
    >,
}

/// Unforgeable proof that committed-outbox resolution succeeded.
///
/// The wrapped binding still distinguishes exact `Some` from honest `None`.
/// No receipt path can manufacture this seal from a denied resolution.
pub struct WorthQueryCommittedDispatchOutboxReceiptSeal {
    binding: Option<WorthQueryCommittedDispatchOutboxBinding>,
}



impl WorthQueryCommittedDispatchOutboxResolution {
    pub(in crate::domain_computation::primary_graph::provider) fn from_commit(
        pending: Option<&WorthQueryPendingDispatchOutbox>,
        committed: &CommitResult,
    ) -> Self {
        Self {
            binding: WorthQueryCommittedDispatchOutboxBinding::from_commit(pending, committed),
        }
    }

    /// Sole fallible decision consumed by receipt assembly.
    pub(in crate::domain_computation::primary_graph) fn seal_for_receipt(
        &self,
    ) -> Result<
        WorthQueryCommittedDispatchOutboxReceiptSeal,
        WorthQueryCommittedDispatchOutboxBindingDenial,
    > {
        match &self.binding {
            Ok(binding) => Ok(WorthQueryCommittedDispatchOutboxReceiptSeal {
                binding: binding.clone(),
            }),
            Err(denial) => Err(*denial),
        }
    }
}

impl WorthQueryCommittedDispatchOutboxReceiptSeal {
    pub fn binding(&self) -> Option<&WorthQueryCommittedDispatchOutboxBinding> {
        self.binding.as_ref()
    }

    pub fn into_binding(self) -> Option<WorthQueryCommittedDispatchOutboxBinding> {
        self.binding
    }
}


mod court_doors_for_test;
