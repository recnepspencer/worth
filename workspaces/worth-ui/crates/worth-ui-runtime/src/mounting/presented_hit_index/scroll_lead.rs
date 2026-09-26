//! Leading hit rows to a Scroll pose the host displays before mounted
//! geometry can commit it.
//!
//! A witness can display an accepted Scroll sample while a presentation
//! attempt is in flight, and mounted geometry refuses the pose until that
//! attempt ends. Paint already shows the content where the sample put it, so
//! a pointer has to reach it there too. A lead moves the rows by the pose's
//! translations without committing them: the next lead replaces it, and the
//! pose that finally commits, or a republished row, retires it. A frame that
//! replaces the displayed one while the settle is owed shows the same sample,
//! so it takes the leads of the rows it reuses.
use super::*;

#[cfg(test)]
mod tests;

/// Whether a pose moving hit rows is one mounted geometry committed, or one
/// the host displays ahead of that commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::mounting) enum UiHitScrollStanding {
    /// Mounted geometry committed the pose; the rows keep it.
    Committed,
    /// The host displays the pose, and mounted geometry owes its commit;
    /// the rows lead to it until a commit or a later lead replaces it.
    Leading,
}

impl Record {
    /// This record led by `lead` past its committed poses, replacing any
    /// lead it held.
    pub(super) fn led(self, lead: UiHitScrollMove) -> Self {
        Self {
            scroll_lead: lead,
            ..self
        }
        .projected(self.unscrolled, self.scroll_translation)
    }
}

impl UiPresentedHitIndex {
    /// Move `binding`'s rows by one pose's translations as `standing` says.
    pub(in crate::mounting) fn move_by_scroll(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        translations: &[(UiMountedInstanceIdentity, UiHitScrollMove)],
        standing: UiHitScrollStanding,
    ) -> UiHitTestSpatialWork {
        match standing {
            UiHitScrollStanding::Committed => self.apply_scroll_translations(binding, translations),
            UiHitScrollStanding::Leading => self.lead_scroll_translations(binding, translations),
        }
    }

    /// Lead `binding`'s rows by one displayed pose's translations, measured
    /// from the poses committed so far, and retire every lead an earlier
    /// pose left on the rest.
    pub(super) fn lead_scroll_translations(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        leads: &[(UiMountedInstanceIdentity, UiHitScrollMove)],
    ) -> UiHitTestSpatialWork {
        let mut work = UiHitTestSpatialWork::default();
        let leading = leads
            .iter()
            .map(|(instance, _)| *instance)
            .collect::<std::collections::BTreeSet<_>>();
        let retired = self
            .scroll_leads
            .iter()
            .copied()
            .filter(|instance| !leading.contains(instance))
            .collect::<Vec<_>>();
        let moves = retired
            .into_iter()
            .map(|instance| (instance, UiHitScrollMove::none()))
            .chain(leads.iter().copied());
        for (instance, lead) in moves {
            let (record, probes) = self.rows.get_with_probes(&instance);
            work.map_key_probes += probes;
            let Some(record) = record.copied() else {
                self.scroll_leads.remove(&instance);
                continue;
            };
            if record.base.mounted().binding() != binding {
                continue;
            }
            let next = record.led(lead);
            if !same_place(next.effective, record.effective) {
                work.scroll_rows_displaced += 1;
            }
            self.store(instance, record, next, &mut work);
            self.note_scroll_lead(instance, lead);
        }
        work
    }

    /// Take the leads `previous` holds on the rows this index reuses from
    /// it. Mounted geometry cannot commit a pose while the frame that holds
    /// this index is in flight, so each lead still measures from the poses
    /// its row carries here.
    pub(in crate::mounting) fn inherit_scroll_leads(
        &mut self,
        previous: &Self,
        binding: UiSurfaceBindingGeneration,
    ) -> UiHitTestSpatialWork {
        let mut work = UiHitTestSpatialWork::default();
        for instance in previous.scroll_leads.iter() {
            let (old, old_probes) = previous.rows.get_with_probes(instance);
            let (new, new_probes) = self.rows.get_with_probes(instance);
            work.map_key_probes += old_probes + new_probes;
            let (Some(old), Some(new)) = (old.copied(), new.copied()) else {
                continue;
            };
            // Only the retained source row, including its receipt, carrying
            // the same committed poses proves the lead measures from it.
            if new.base.mounted().binding() != binding
                || old.base != new.base
                || old.scroll_translation != new.scroll_translation
            {
                continue;
            }
            let next = new.led(old.scroll_lead);
            if !same_place(next.effective, new.effective) {
                work.scroll_rows_displaced += 1;
            }
            self.store(*instance, new, next, &mut work);
            self.note_scroll_lead(*instance, old.scroll_lead);
        }
        work
    }

    /// Whether any row here leads past its committed poses.
    pub(in crate::mounting) fn holds_scroll_leads(&self) -> bool {
        !self.scroll_leads.is_empty()
    }

    /// Track whether `instance`'s row holds a lead, so the next pose can
    /// retire it.
    pub(super) fn note_scroll_lead(
        &mut self,
        instance: UiMountedInstanceIdentity,
        lead: UiHitScrollMove,
    ) {
        if lead == UiHitScrollMove::none() {
            self.scroll_leads.remove(&instance);
        } else {
            self.scroll_leads.insert(instance);
        }
    }
}
