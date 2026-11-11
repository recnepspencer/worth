//! Which home checkpoint Store has selected: the generation of the
//! `home.checkpoint` anchored record, read through the home's Store.

pub(crate) use worth_store::physical_runtime::AnchoredRecordName;
pub(crate) use worth_store_application::ReadmittedApplicationHome;

pub(crate) const ANCHOR: &str = "home.checkpoint";

/// The selected checkpoint's generation; 0 when the home has none.
pub(crate) fn generation(home: &ReadmittedApplicationHome) -> u64 {
    generation_from_serving(home.physical_serving_for_test())
}

pub(crate) fn generation_from_serving(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> u64 {
    serving
        .anchored_records()
        .expect("anchored records")
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new(ANCHOR).unwrap(),
            1 << 30,
        )
        .expect("the checkpoint anchor reads")
        .map_or(0, |record| record.version().generation())
}
