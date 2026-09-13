//! Closed descriptive material for publication consumers.

use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitTerminalKind, WorthQueryPrimaryMutationWorkEvidence,
};

/// Execution-owned, non-authoritative description of one commit terminal.
///
/// It intentionally carries no receipt, runtime, branch, session, record, or
/// causal identity and cannot be converted back into commit authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCommitPublicationSource {
    terminal_kind: WorthQueryApplicationCommitTerminalKind,
    mutation_work: Option<WorthQueryPrimaryMutationWorkEvidence>,
    changed_record_count: usize,
    emitted_effect_count: usize,
    publication_work: WorthQueryCanonicalWorkEvidence,
    attempt_resources_released: Option<bool>,
}

impl WorthQueryApplicationCommitPublicationSource {
    pub(super) fn from_receipt(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        Self {
            terminal_kind: receipt.terminal().kind(),
            mutation_work: receipt.mutation_work().cloned(),
            changed_record_count: receipt.changed_record_count(),
            emitted_effect_count: receipt.emitted_effect_count(),
            publication_work: receipt.canonical_work().publication(),
            attempt_resources_released: receipt.terminal().attempt_resources_released(),
        }
    }

    pub const fn terminal_kind(&self) -> WorthQueryApplicationCommitTerminalKind {
        self.terminal_kind
    }

    pub const fn mutation_work(&self) -> Option<&WorthQueryPrimaryMutationWorkEvidence> {
        self.mutation_work.as_ref()
    }

    pub const fn changed_record_count(&self) -> usize {
        self.changed_record_count
    }

    pub const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }

    pub const fn publication_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.publication_work
    }

    pub const fn attempt_resources_released(&self) -> Option<bool> {
        self.attempt_resources_released
    }
}
