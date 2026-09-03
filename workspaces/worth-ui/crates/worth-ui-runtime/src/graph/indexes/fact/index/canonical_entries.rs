use super::super::UiGraphFactIndexEntry;

pub(super) fn canonical_entries(
    mut entries: Vec<UiGraphFactIndexEntry>,
) -> Box<[UiGraphFactIndexEntry]> {
    entries.sort_by(|left, right| {
        left.consumer_key()
            .cmp(right.consumer_key())
            .then_with(|| left.consumer().cmp(&right.consumer()))
            .then_with(|| {
                left.consumption_relation()
                    .cmp(right.consumption_relation())
            })
    });
    entries.dedup();
    entries.into_boxed_slice()
}
