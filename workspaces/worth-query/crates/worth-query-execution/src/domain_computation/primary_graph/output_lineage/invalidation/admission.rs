use worth_relational::facade::mvcc::{CompanionPreflightStop, PublicationCompanionPreflight};

use super::index_capacity;

/// Both publication preparation and same-position derived edits use this
/// boundary. Capacity forecasts are admitted before a persistent path is edited.
pub(super) trait IndexAdmission {
    #[track_caller]
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop>;
    fn bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop>;
    /// Only a keyed descent's path is navigation, bounded by declared
    /// operations times H. Validation, visited entries and key-payload
    /// comparisons in the selected branch and scope remain logical work.
    fn navigation(&mut self, units: u64) -> Result<(), CompanionPreflightStop>;

    #[track_caller]
    fn key_read(
        &mut self,
        key: &super::fact_key::FactPostingKey,
        entries: usize,
    ) -> Result<(), CompanionPreflightStop> {
        if entries == 0 {
            // A vacant root visits its empty slot but compares no stored key.
            // Key construction and retained backing are admitted by the owner.
            return self.work(1);
        }
        let navigation = index_capacity::ordered_navigation_work(entries)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        self.key_navigation(key, navigation)
    }

    #[track_caller]
    fn key_navigation(
        &mut self,
        key: &super::fact_key::FactPostingKey,
        navigation: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.work(key_traversal_work(key))?;
        let payload = key
            .comparison_work_bound()
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        // Variable key payload is logical work; only the selected path is
        // navigation. These indexes are already selected by branch and scope.
        self.work(
            payload
                .saturating_sub(1)
                .checked_mul(navigation)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        self.navigation(navigation)
    }

    #[track_caller]
    fn key_edit<K, V>(
        &mut self,
        key: &super::fact_key::FactPostingKey,
        entries: usize,
    ) -> Result<(), CompanionPreflightStop> {
        self.key_read(key, entries)?;
        self.bytes(
            index_capacity::ordered_edit_bytes::<K, V>(entries)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )
    }

    #[track_caller]
    fn ordered_read(&mut self, entries: usize) -> Result<(), CompanionPreflightStop> {
        // One declared unit per ordered operation; its path is navigation.
        self.work(1)?;
        self.navigation(
            index_capacity::ordered_navigation_work(entries)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )
    }

    #[track_caller]
    fn ordered_edit<K, V>(&mut self, entries: usize) -> Result<(), CompanionPreflightStop> {
        self.ordered_read(entries)?;
        self.bytes(
            index_capacity::ordered_edit_bytes::<K, V>(entries)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )
    }

    fn ordered_remove<K, V>(&mut self, entries: usize) -> Result<(), CompanionPreflightStop> {
        self.work(1)?;
        self.navigation(
            index_capacity::ordered_removal_work(entries)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        self.bytes(
            index_capacity::ordered_edit_bytes::<K, V>(entries)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )
    }

    fn key_remove<K, V>(
        &mut self,
        key: &super::fact_key::FactPostingKey,
        entries: usize,
    ) -> Result<(), CompanionPreflightStop> {
        let navigation = index_capacity::ordered_removal_work(entries)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        self.key_navigation(key, navigation)?;
        self.bytes(
            index_capacity::ordered_edit_bytes::<K, V>(entries)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )
    }
}

/// A key's own path traversal is logical work on every admission; only the
/// payload comparisons along the physical index path depend on its height.
pub(super) fn key_traversal_work(key: &super::fact_key::FactPostingKey) -> u64 {
    use super::fact_key::FactPostingKey as Key;
    match key {
        Key::FieldRevision { path, .. }
        | Key::PredicateField { path, .. }
        | Key::IndexMembership { path, .. } => path.fields().len() as u64 + 1,
        Key::EntityLifecycle(_)
        | Key::EntityKind(_)
        | Key::RelationKind(_)
        | Key::AspectRevision { .. }
        | Key::RelationMembership { .. }
        | Key::Adjacency { .. }
        | Key::IndexDefinition(_) => 1,
    }
}

impl IndexAdmission for PublicationCompanionPreflight<'_> {
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        self.claim_work(visits)
    }
    fn bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        self.claim_bytes(bytes)
    }
    /// Relational's publication budget admits declared work and bytes; the
    /// physical path observes interruption without spending either.
    fn navigation(&mut self, _units: u64) -> Result<(), CompanionPreflightStop> {
        self.checkpoint()
    }
}

#[cfg(test)]
mod tests;
