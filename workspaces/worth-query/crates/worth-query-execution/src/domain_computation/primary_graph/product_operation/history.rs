//! Query audience binding for bounded World-owned product history.

use std::num::NonZeroUsize;

use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositeComponentChangePosture, CompositeRuntimeWorldCommit,
    ProductBranchHistoryTraversal,
};

use super::{WorthQueryApplicationProductBranches, WorthQuerySelectedProductOperation};
use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// A bounded owner-issued ancestry page for one Query product occurrence.
/// The page keeps its exact World history and component obligations alive.
pub struct WorthQueryProductHistory<'runtime, Schema> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    history: ProductBranchHistoryTraversal,
}

/// Descriptive view of one commit retained by a `WorthQueryProductHistory`.
pub struct WorthQueryProductHistoryEntry<'history> {
    index: usize,
    commit: &'history CompositeRuntimeWorldCommit,
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryApplicationProductBranches<'runtime, Schema> {
    pub fn history(
        &self,
        branch: WorthQueryProductBranch,
        maximum: NonZeroUsize,
    ) -> Result<WorthQueryProductHistory<'runtime, Schema>, WorthQueryProductBranchAdmissionDenial>
    {
        let history = self
            .application
            .product_runtime
            .trace_product_history(branch, maximum)?;
        Ok(WorthQueryProductHistory {
            application: self.application,
            history,
        })
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryProductHistory<'runtime, Schema> {
    pub fn visited_count(&self) -> usize {
        self.history.visited_count()
    }

    pub fn is_complete(&self) -> bool {
        self.history.is_complete()
    }

    pub fn next_parent_commit(&self) -> Option<&CompositeCommitIdentity> {
        self.history.next_parent()
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = WorthQueryProductHistoryEntry<'_>> {
        self.history
            .commits()
            .enumerate()
            .map(|(index, commit)| WorthQueryProductHistoryEntry { index, commit })
    }

    pub fn continue_ancestry(
        &self,
        maximum: NonZeroUsize,
    ) -> Result<Self, WorthQueryProductBranchAdmissionDenial> {
        let history = self
            .application
            .product_runtime
            .continue_product_history(&self.history, maximum)?;
        Ok(Self {
            application: self.application,
            history,
        })
    }

    pub fn select(
        &self,
        entry: &WorthQueryProductHistoryEntry<'_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        let expected = self
            .history
            .commits()
            .nth(entry.index)
            .ok_or(WorthQueryProductBranchAdmissionDenial::ObservationRejected)?;
        if !std::ptr::eq(entry.commit, expected) {
            return Err(WorthQueryProductBranchAdmissionDenial::ObservationRejected);
        }
        let product = self
            .application
            .product_runtime
            .admit_product_history_entry(&self.history, entry.index)?;
        self.application.on_product(product)
    }
}

impl WorthQueryProductHistoryEntry<'_> {
    pub fn selected_commit(&self) -> &CompositeCommitIdentity {
        self.commit.identity()
    }

    pub fn relational_posture(&self) -> CompositeComponentChangePosture {
        self.commit.relational_change()
    }

    pub fn signal_posture(&self) -> CompositeComponentChangePosture {
        self.commit.signal_change()
    }
}
