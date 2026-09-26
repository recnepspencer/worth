//! Carry accepted Scroll displacement only across an unchanged source hit row.
use super::*;

#[cfg(test)]
mod tests;

impl Record {
    /// The record of `base` as its frame publishes it, before any Motion or
    /// pose moves it.
    pub(super) fn published(base: UiPresentedHitTestRow) -> Self {
        Self {
            base,
            unscrolled: None,
            effective: None,
            scroll_translation: UiHitScrollMove::none(),
            scroll_lead: UiHitScrollMove::none(),
        }
        .projected(Some(base), UiHitScrollMove::none())
    }

    /// This record with its row projected to `unscrolled` and moved by
    /// `scroll_translation`, every Scroll pose committed since it was
    /// published, then by any lead it holds. The move is one step from the
    /// projection, as the interaction basis takes it, so both lanes land the
    /// row on one rect.
    pub(super) fn projected(
        self,
        unscrolled: Option<UiPresentedHitTestRow>,
        scroll_translation: UiHitScrollMove,
    ) -> Self {
        let displayed = scroll_translation.then(self.scroll_lead);
        Self {
            unscrolled,
            // A pose can leave the row's ancestors sharing no coverage, and
            // then nothing of it can be reached.
            effective: unscrolled
                .map(|row| row.scroll_translated(displayed))
                .filter(|row| row.ancestor_reach() != crate::mounting::UiHitAncestorReach::Nowhere),
            scroll_translation,
            ..self
        }
    }
}

impl UiPresentedHitIndex {
    /// How far the Scroll poses displayed since `instance`'s row was
    /// published have moved it, in the binding the row belongs to: every
    /// committed pose and any lead past them. `None` when the index holds no
    /// row for `instance` in that binding.
    pub(in crate::mounting) fn displayed_scroll_translation(
        &self,
        binding: UiSurfaceBindingGeneration,
        instance: UiMountedInstanceIdentity,
    ) -> (Option<UiHitScrollMove>, usize) {
        let (record, probes) = self.rows.get_with_probes(&instance);
        let translation = record
            .filter(|record| record.base.mounted().binding() == binding)
            .map(|record| record.scroll_translation.then(record.scroll_lead));
        (translation, probes)
    }

    /// The ancestor clips `instance`'s row was published inside. `None`
    /// when the frame published no row for it.
    pub(in crate::mounting) fn published_ancestor_clip(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiHitAncestorClip> {
        self.rows
            .get(&instance)
            .map(|record| record.base.published_ancestor_clip())
    }

    pub(in crate::mounting) fn inherit_accepted_scroll(
        &mut self,
        previous: &Self,
        binding: UiSurfaceBindingGeneration,
    ) -> UiHitTestSpatialWork {
        let (changed, comparison) = self.rows.changed_keys_with_work(&previous.rows);
        let mut work = UiHitTestSpatialWork::default();
        work.succession_cursor_steps += comparison.cursor_steps();
        for instance in changed {
            let (old, old_probes) = previous.rows.get_with_probes(&instance);
            let (new, new_probes) = self.rows.get_with_probes(&instance);
            work.map_key_probes += old_probes + new_probes;
            let (Some(old), Some(new)) = (old.copied(), new.copied()) else {
                continue;
            };
            if new.base.mounted().binding() != binding
                || old.scroll_translation == new.scroll_translation
                // A fresh layout can return to exactly the old rectangle. Only
                // the retained source row, including its receipt, proves reuse.
                || old.base != new.base
            {
                continue;
            }
            let next = new.projected(new.unscrolled, old.scroll_translation);
            self.update_partition(new.base, new.effective, next.effective, 0, &mut work);
            work.scroll_rows_displaced += 1;
            record_map(&mut work, self.rows.insert_with_work(instance, next));
        }
        work
    }
}
