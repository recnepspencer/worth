use super::family_chain::{
    compose_consumed_projection_fact_family_digest, FACT_FAMILY_CHUNK_WIDTH,
};

fn entry(index: usize) -> String {
    format!("fact-entry-{index}")
}

fn digest(family: &'static str, entries: impl IntoIterator<Item = String>) -> String {
    compose_consumed_projection_fact_family_digest(family, entries)
}

#[test]
fn empty_fact_families_are_explicit_and_family_separated() {
    let entity = digest("entity_identity", std::iter::empty());
    let membership = digest("membership", std::iter::empty());

    assert_ne!(entity, membership);
    assert_eq!(entity, digest("entity_identity", std::iter::empty()));
    assert_ne!(
        digest("entity_identity", [entry(0)]),
        digest("membership", [entry(0)])
    );
    assert_ne!(entity, digest("entity_identity", [entry(0)]));
}

#[test]
fn fact_family_digest_binds_order_count_and_omission() {
    let ordered = digest("entity_identity", [entry(0), entry(1), entry(2)]);
    let reordered = digest("entity_identity", [entry(1), entry(0), entry(2)]);
    let omitted = digest("entity_identity", [entry(0), entry(1)]);

    assert_ne!(ordered, reordered);
    assert_ne!(ordered, omitted);
}

#[test]
fn fact_family_digest_changes_across_chunk_edges_and_late_rows() {
    let edge = (0..FACT_FAMILY_CHUNK_WIDTH).map(entry).collect::<Vec<_>>();
    let mut beyond_edge = edge.clone();
    beyond_edge.push(entry(FACT_FAMILY_CHUNK_WIDTH));
    let mut changed_late_row = beyond_edge.clone();
    changed_late_row[FACT_FAMILY_CHUNK_WIDTH] = "changed-late-row".to_string();

    assert_ne!(
        digest("display_field", edge),
        digest("display_field", beyond_edge)
    );
    assert_ne!(
        digest("display_field", (0..=FACT_FAMILY_CHUNK_WIDTH).map(entry)),
        digest("display_field", changed_late_row)
    );
}

#[test]
fn fact_family_digest_consumes_every_entry_beyond_the_canonical_flat_limit() {
    let fact_count = 4_097;
    let complete = digest("entity_identity", (0..fact_count).map(entry));
    let changed_last = digest(
        "entity_identity",
        (0..fact_count).map(|index| {
            if index + 1 == fact_count {
                "changed-final-entry".to_string()
            } else {
                entry(index)
            }
        }),
    );
    let changed_first = digest(
        "entity_identity",
        (0..fact_count).map(|index| {
            if index == 0 {
                "changed-first-entry".to_string()
            } else {
                entry(index)
            }
        }),
    );
    let omitted_last = digest("entity_identity", (0..fact_count - 1).map(entry));

    assert_ne!(complete, changed_last);
    assert_ne!(complete, changed_first);
    assert_ne!(complete, omitted_last);
}
