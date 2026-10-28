//! Which home checkpoint Store has selected: the generation of the
//! `relational.checkpoint` anchored record, read through the home's Store.

use worth_store::physical_runtime::AnchoredRecordName;
use worth_store_relational::RelationalStoreHome;

pub(crate) const ANCHOR: &str = "relational.checkpoint";

/// The selected checkpoint's generation; 0 when the home has none.
pub(crate) fn generation(home: &RelationalStoreHome) -> u64 {
    home.certification_serving()
        .anchored_records()
        .expect("anchored records")
        .read(AnchoredRecordName::new(ANCHOR).unwrap(), 1 << 30)
        .expect("the checkpoint anchor reads")
        .map_or(0, |record| record.version().generation())
}
