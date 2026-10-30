//! Eligibility and the single effect path for retiring an installed interior.
use super::super::support::{detach_installed, effective_parent};
use super::super::{lock_index, CompositeHistoryCatalogState};
use super::{installed, SplicedParent};
use crate::identity::CompositeCommitIdentity;

/// Assessed while the caller holds the catalog lock; construction is private
/// to the entry owner, and application consumes the exact selected child/span.
pub(super) struct EligibleInterior {
    commit: CompositeCommitIdentity,
    child: CompositeCommitIdentity,
    child_span: usize,
    parent: Option<(CompositeCommitIdentity, usize)>,
}

impl EligibleInterior {
    pub(super) fn assess(
        state: &CompositeHistoryCatalogState,
        commit: &CompositeCommitIdentity,
    ) -> Option<Self> {
        let entry = installed(state, commit)?;
        if entry
            .publication
            .as_ref()
            .is_some_and(|publication| publication.awaits_delivery())
        {
            return None;
        }
        let record = lock_index(&state.reachability)
            .lookup(commit)
            .expect("an installed commit has its reachability record");
        if record.direct_protections() != 0 {
            return None;
        }
        let child = record.sole_installed_child()?.clone();
        let child_entry = installed(state, &child).expect("a reverse child is installed");
        let (selected_parent, child_span) = effective_parent(state, child_entry.commit())
            .expect("an installed child has its effective parent");
        assert_eq!(
            &selected_parent, commit,
            "reverse and canonical effective edges agree"
        );
        let parent = effective_parent(state, entry.commit())
            .filter(|(parent, _)| installed(state, parent).is_some());
        Some(Self {
            commit: commit.clone(),
            child,
            child_span,
            parent,
        })
    }

    pub(super) fn child(&self) -> &CompositeCommitIdentity {
        &self.child
    }

    pub(super) fn retire(
        self,
        state: &mut CompositeHistoryCatalogState,
    ) -> std::sync::Arc<std::sync::OnceLock<super::super::CompositeHistoryCatalogEntry>> {
        lock_index(&state.reachability).transfer_sole_child(&self.commit, &self.child);
        let (entry, _) = detach_installed(state, &self.commit);
        match self.parent {
            Some((parent, span)) => {
                state.spliced.insert(
                    self.child,
                    SplicedParent {
                        parent,
                        generations: self.child_span.saturating_add(span),
                    },
                );
            }
            None => {
                state.spliced.remove(&self.child);
                state.root = Some(self.child);
            }
        }
        entry
    }
}
