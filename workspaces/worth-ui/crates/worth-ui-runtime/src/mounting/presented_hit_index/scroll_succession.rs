//! Carry accepted Scroll displacement only across an unchanged source hit row.
use super::*;

#[cfg(test)]
mod tests;

impl UiPresentedHitIndex {
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
            let delta = [
                old.scroll_translation[0] - new.scroll_translation[0],
                old.scroll_translation[1] - new.scroll_translation[1],
            ];
            let effective = new.effective.map(|row| row.scroll_translated(delta));
            self.update_partition(new.base, new.effective, effective, 0, &mut work);
            work.scroll_rows_displaced += 1;
            record_map(
                &mut work,
                self.rows.insert_with_work(
                    instance,
                    Record {
                        effective,
                        scroll_translation: old.scroll_translation,
                        ..new
                    },
                ),
            );
        }
        work
    }
}
