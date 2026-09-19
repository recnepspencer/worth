use super::super::scope::{scope_encoder, seal};
use crate::WorthQueryEvidenceTag;

pub(super) const FACT_FAMILY_CHUNK_WIDTH: usize = 1_024;

pub(super) fn compose_consumed_projection_fact_family_digest(
    fact_family: &'static str,
    entries: impl IntoIterator<Item = String>,
) -> String {
    let mut terminal_chunk_digest = compose_fact_family_genesis_digest(fact_family);
    let mut chunk_entries = Vec::with_capacity(FACT_FAMILY_CHUNK_WIDTH);
    let mut entry_count = 0usize;
    let mut chunk_count = 0usize;

    for entry in entries {
        chunk_entries.push(entry);
        entry_count += 1;
        if chunk_entries.len() == FACT_FAMILY_CHUNK_WIDTH {
            terminal_chunk_digest = compose_fact_family_chunk_digest(
                fact_family,
                chunk_count,
                entry_count,
                &terminal_chunk_digest,
                chunk_entries.drain(..),
            );
            chunk_count += 1;
        }
    }

    if !chunk_entries.is_empty() {
        terminal_chunk_digest = compose_fact_family_chunk_digest(
            fact_family,
            chunk_count,
            entry_count,
            &terminal_chunk_digest,
            chunk_entries,
        );
        chunk_count += 1;
    }

    seal(
        scope_encoder("consumed_projection_fact_family_root_v2")
            .field_shape(WorthQueryEvidenceTag::new("fact_family"), fact_family)
            .field_usize(WorthQueryEvidenceTag::new("entry_count"), entry_count)
            .field_usize(WorthQueryEvidenceTag::new("chunk_count"), chunk_count)
            .field_shape(
                WorthQueryEvidenceTag::new("terminal_chunk_digest"),
                terminal_chunk_digest,
            ),
    )
}

fn compose_fact_family_genesis_digest(fact_family: &'static str) -> String {
    seal(
        scope_encoder("consumed_projection_fact_family_genesis_v2")
            .field_shape(WorthQueryEvidenceTag::new("fact_family"), fact_family),
    )
}

fn compose_fact_family_chunk_digest(
    fact_family: &'static str,
    chunk_ordinal: usize,
    entry_count_through_chunk: usize,
    predecessor_chunk_digest: &str,
    entries: impl IntoIterator<Item = String>,
) -> String {
    seal(
        scope_encoder("consumed_projection_fact_family_chunk_v2")
            .field_shape(WorthQueryEvidenceTag::new("fact_family"), fact_family)
            .field_usize(WorthQueryEvidenceTag::new("chunk_ordinal"), chunk_ordinal)
            .field_usize(
                WorthQueryEvidenceTag::new("entry_count_through_chunk"),
                entry_count_through_chunk,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("predecessor_chunk_digest"),
                predecessor_chunk_digest,
            )
            .field_value_sequence(WorthQueryEvidenceTag::new("fact_entry"), entries),
    )
}
