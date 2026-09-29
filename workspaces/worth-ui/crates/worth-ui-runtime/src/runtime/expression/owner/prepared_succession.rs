use std::collections::BTreeSet;
use std::sync::Arc;

use super::operand_binding::UiExpressionInputs;
use super::{UiExpressionRuntimeState, UiExpressionSettlement};
use crate::runtime::expression::UiExpressionCatalog;
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

/// A generation succession of the expression owner, computed from the owner
/// without changing it. A reader of conditions at the successor generation
/// reads them through it before the successor is active; committing it
/// installs exactly the state those readers saw, and dropping it leaves the
/// owner as it was.
#[must_use = "a prepared succession changes nothing until it is committed"]
pub(crate) struct UiPreparedExpressionSuccession {
    basis: UiExpressionSuccessionBasis,
    /// The successor, or `None` when the owner already follows that
    /// generation and catalog.
    successor: Option<UiExpressionRuntimeState>,
}

/// What the preparation read: the owner's generation and catalog, the
/// revision of every application fact and the frame whose projection inputs
/// were current.
struct UiExpressionSuccessionBasis {
    generation: WorthUiActiveApplicationGenerationIdentity,
    catalog: Arc<UiExpressionCatalog>,
    facts: Box<[u64]>,
    frame: Option<crate::mounting::UiMountedFrameIdentity>,
}

impl UiExpressionRuntimeState {
    /// Computes this owner's move to the generation `inputs` names, against
    /// the catalog that generation installs, reading the owners of `inputs`
    /// as they stand now. Its records, generation and catalog stay as they
    /// are until the result is committed; the work of computing it counts
    /// now, whether or not it commits.
    pub(crate) fn prepare_succession(
        &mut self,
        catalog: &Arc<UiExpressionCatalog>,
        inputs: &UiExpressionInputs<'_>,
    ) -> UiPreparedExpressionSuccession {
        let follows = self.follows(inputs.generation) && Arc::ptr_eq(&self.catalog, catalog);
        let successor = (!follows).then(|| self.successor(catalog, inputs));
        if let Some(successor) = &successor {
            self.counters = self.counters.absorbing(successor.counters);
        }
        UiPreparedExpressionSuccession {
            basis: UiExpressionSuccessionBasis {
                generation: self.generation.clone(),
                catalog: Arc::clone(&self.catalog),
                facts: inputs.facts.revisions(),
                frame: inputs.mounted.current_projection_frame(),
            },
            successor,
        }
    }

    /// Installs the successor `prepared` holds, with no evaluation of its
    /// own, and makes it the owner of the generation `inputs` names. It is
    /// the only place the owner's generation changes.
    ///
    /// A writer that commits after effects may find that the owners the
    /// preparation read have moved since: an application fact updated while
    /// a detached publication was in flight, or the frame that publication
    /// itself made current. Each is settled into the successor through the
    /// dependency index, as the owner settles any update or frame.
    ///
    /// The settlement names each expression whose committed outcome differs
    /// from the outcome the replaced owner held for the same identity at the
    /// moment of the swap, including an identity the replaced owner does not
    /// install. That owner is what the consumers last observed: it may itself
    /// have settled the published frame before the swap.
    pub(crate) fn commit_succession(
        &mut self,
        prepared: UiPreparedExpressionSuccession,
        catalog: &Arc<UiExpressionCatalog>,
        inputs: &UiExpressionInputs<'_>,
    ) -> UiExpressionSettlement {
        let UiPreparedExpressionSuccession { basis, successor } = prepared;
        assert!(
            basis.generation == self.generation && Arc::ptr_eq(&basis.catalog, &self.catalog),
            "a succession commits onto the owner it was prepared from"
        );
        let Some(successor) = successor else {
            return UiExpressionSettlement::default();
        };
        assert!(
            successor.follows(inputs.generation) && Arc::ptr_eq(&successor.catalog, catalog),
            "a succession installs the generation and catalog it was prepared for"
        );
        let replaced = std::mem::replace(self, successor);
        self.counters = replaced.counters;
        // The catch-up settles into the successor; what the consumers must
        // re-observe is measured once, against the replaced owner, below.
        for receipt in inputs.facts.updated_since(&basis.facts) {
            let _caught_up = self.invalidate_application(&receipt, inputs);
        }
        if inputs.mounted.current_projection_frame() != basis.frame {
            let _caught_up = self.invalidate_published_frame(inputs);
        }
        UiExpressionSettlement::new(self.changed_since(&replaced))
    }

    /// The slots of this owner whose outcome differs from `replaced`'s
    /// outcome for the same identity, with no evaluation: one lookup per
    /// installed slot.
    fn changed_since(
        &self,
        replaced: &Self,
    ) -> BTreeSet<crate::runtime::expression::UiExpressionSlot> {
        self.catalog
            .slot_count()
            .slots()
            .filter(|slot| {
                let current = self.records.get(*slot).map(|record| &record.outcome);
                let prior = self
                    .catalog
                    .expression(*slot)
                    .and_then(|expression| replaced.catalog.slot_of(expression.identity()))
                    .and_then(|prior| replaced.records.get(prior))
                    .map(|record| &record.outcome);
                current != prior
            })
            .collect()
    }
}

impl UiPreparedExpressionSuccession {
    /// The owner a reader at the successor generation reads before the
    /// commit: the prepared successor, or `current` when that already
    /// follows it.
    pub(crate) fn successor_owner<'owner>(
        &'owner self,
        current: &'owner UiExpressionRuntimeState,
    ) -> &'owner UiExpressionRuntimeState {
        self.successor.as_ref().unwrap_or(current)
    }
}
