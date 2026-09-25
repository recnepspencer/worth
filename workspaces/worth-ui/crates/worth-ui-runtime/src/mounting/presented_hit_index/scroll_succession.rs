//! Carry accepted Scroll displacement only across an unchanged source hit row.
use super::*;

#[cfg(test)]
mod tests;

impl Record {
    /// This record with its row projected to `unscrolled` and moved by
    /// `scroll_translation`, every Scroll pose committed since it was
    /// published. The move is one step from the projection, as the
    /// interaction basis takes it, so both lanes land the row on one rect.
    pub(super) fn projected(
        self,
        unscrolled: Option<UiPresentedHitTestRow>,
        scroll_translation: super::UiScrollPoseShift,
    ) -> Self {
        Self {
            unscrolled,
            effective: unscrolled.map(|row| row.scroll_translated(scroll_translation)),
            scroll_translation,
            ..self
        }
    }
}

impl UiPresentedHitIndex {
    /// How far the Scroll poses committed since `instance`'s row was
    /// published have moved it, in the binding the row belongs to. `None`
    /// when the index holds no row for `instance` in that binding.
    pub(in crate::mounting) fn committed_scroll_translation(
        &self,
        binding: UiSurfaceBindingGeneration,
        instance: UiMountedInstanceIdentity,
    ) -> (Option<super::UiScrollPoseShift>, usize) {
        let (record, probes) = self.rows.get_with_probes(&instance);
        let translation = record
            .filter(|record| record.base.mounted().binding() == binding)
            .map(|record| record.scroll_translation);
        (translation, probes)
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
