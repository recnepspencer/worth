//! Validate event-page coverage and routing declarations without reading payloads.
use super::manifest_capture::corrupt;
use crate::durability::data::*;
use std::collections::BTreeSet;

pub(super) fn validate(
    manifest: &RelationalCheckpointManifestDraft,
) -> Result<(), DurabilityError> {
    let mut members = BTreeSet::new();
    let mut event_ids = BTreeSet::new();
    for page in &manifest.lineage_events {
        if page.first_node == 0
            || page.last_node < page.first_node
            || page.last_node >= u64::MAX - 1
            || page.event_ids.is_empty()
            || page.event_ids.len() > LINEAGE_EVENT_PAGE_ROWS
            || page.event_ids.windows(2).any(|pair| pair[0] >= pair[1])
            || page
                .event_ids
                .iter()
                .any(|id| *id == 0 || *id >= u64::MAX - 1 || !event_ids.insert(*id))
            || page.routes.is_empty()
            || page.routes.windows(2).any(|pair| pair[0] >= pair[1])
            || page.routes.iter().any(|route| {
                route.branch_id.0.is_empty()
                    || !((page.first_node - 1) / LINEAGE_NODE_RANGE_WIDTH
                        ..=(page.last_node - 1) / LINEAGE_NODE_RANGE_WIDTH)
                        .contains(&route.node_range)
            })
            || page.member.length == 0
            || page.member.length > LINEAGE_EVENT_PAGE_BYTES
            || !members.insert(page.member)
        {
            return Err(corrupt("invalid lineage event page directory"));
        }
    }
    if !covers_selected_publication_basis(
        &event_ids,
        &manifest.lineage_basis.canonical_published_event_ids,
        manifest.lineage_basis.published_lineage_event_count,
    ) {
        return Err(corrupt(
            "lineage event pages do not cover the selected publication basis",
        ));
    }
    Ok(())
}

impl super::CapturedManifest {
    pub(crate) fn validate_lineage_events(&self) -> Result<(), DurabilityError> {
        super::manifest_validation::validate_pages(&self.manifest.symbols)?;
        super::manifest_validation::validate_pages(&self.manifest.lineage)?;
        validate(&self.manifest)
    }
}

fn covers_selected_publication_basis(
    event_ids: &BTreeSet<u64>,
    canonical_ids: &[u64],
    count: usize,
) -> bool {
    let expected = canonical_ids.iter().copied().collect::<BTreeSet<_>>();
    expected.len() == canonical_ids.len() && expected.len() == count && event_ids == &expected
}

#[cfg(test)]
mod coverage {
    use super::covers_selected_publication_basis as covers;
    use std::collections::BTreeSet;
    #[test]
    fn lineage_event_directory_coverage_requires_each_selected_published_event() {
        assert!(!covers(&BTreeSet::from([1, 3]), &[1, 2, 3], 3));
    }
    #[test]
    fn lineage_event_directory_coverage_refuses_extra_and_duplicate_selected_ids() {
        assert!(!covers(&BTreeSet::from([1, 2, 3, 4]), &[1, 2, 3], 3));
        assert!(!covers(&BTreeSet::from([1, 2]), &[1, 2, 2], 3));
        assert!(!covers(&BTreeSet::from([1, 2, 3]), &[1, 2, 3], 4));
    }
    #[test]
    fn lineage_event_directory_coverage_accepts_exact_canonical_ids() {
        assert!(covers(&BTreeSet::from([1, 2, 3]), &[1, 2, 3], 3));
    }
}
