//! A root hint cannot retain a publication that the same successor unroutes.

use std::collections::BTreeSet;

use worth_store_physical_format::{
    DerivedFamilyRootDirectoryBinding, IndexedThroughBlobPublication, PersistedRecordIdentity,
};

pub(super) fn surviving_latest_publication(
    current: Option<IndexedThroughBlobPublication>,
    update: Option<IndexedThroughBlobPublication>,
    drops: &BTreeSet<PersistedRecordIdentity>,
) -> Option<IndexedThroughBlobPublication> {
    // There is no authenticated predecessor hint to fall back to when the
    // latest publication is dropped. Clearing the hint is safer than keeping
    // an unrouted identity; selected routes remain the visibility authority.
    update
        .or(current)
        .filter(|value| !drops.contains(&value.record()))
}

pub(super) fn surviving_directory_binding(
    current: Option<DerivedFamilyRootDirectoryBinding>,
    update: Option<DerivedFamilyRootDirectoryBinding>,
    drops: &BTreeSet<PersistedRecordIdentity>,
) -> Option<DerivedFamilyRootDirectoryBinding> {
    // The directory payload owns its watermark. If either its record or the
    // indexed publication is un-routed, retaining the root hint would select
    // an untrustworthy directory; the record can remain derived residue.
    update.or(current).filter(|binding| {
        !drops.contains(&binding.directory_record())
            && binding
                .indexed_through_blob_publication()
                .is_none_or(|publication| !drops.contains(&publication.record()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn released_publication_cannot_remain_a_successor_root_hint() {
        let record = |ordinal| PersistedRecordIdentity::new([7; 16], ordinal).unwrap();
        let old = IndexedThroughBlobPublication::new(3, record(1), [8; 32]).unwrap();
        let next = IndexedThroughBlobPublication::new(4, record(2), [9; 32]).unwrap();
        assert_eq!(
            surviving_latest_publication(Some(old), None, &BTreeSet::from([record(1)])),
            None
        );
        assert_eq!(
            surviving_latest_publication(Some(old), Some(next), &BTreeSet::from([record(1)])),
            Some(next)
        );
        assert_eq!(
            surviving_latest_publication(Some(old), None, &BTreeSet::from([record(3)])),
            Some(old)
        );
        assert_eq!(
            surviving_latest_publication(Some(old), Some(next), &BTreeSet::from([record(2)])),
            None
        );
    }

    #[test]
    fn released_indexed_publication_invalidates_directory_root_hint() {
        let record = |ordinal| PersistedRecordIdentity::new([7; 16], ordinal).unwrap();
        let indexed = IndexedThroughBlobPublication::new(3, record(1), [8; 32]).unwrap();
        let newest = IndexedThroughBlobPublication::new(4, record(2), [9; 32]).unwrap();
        let directory = DerivedFamilyRootDirectoryBinding::new(record(3), Some(indexed));
        // This also covers a surviving newer latest publication: a stale
        // older indexed watermark cannot be kept merely because latest lives.
        assert_eq!(
            surviving_directory_binding(Some(directory), None, &BTreeSet::from([record(1)])),
            None
        );
        assert_eq!(
            surviving_latest_publication(Some(newest), None, &BTreeSet::from([record(1)])),
            Some(newest)
        );
        assert_eq!(
            surviving_directory_binding(Some(directory), None, &BTreeSet::from([record(3)])),
            None
        );
        assert_eq!(
            surviving_directory_binding(Some(directory), None, &BTreeSet::from([record(4)])),
            Some(directory)
        );
    }
}
