//! Rebuild the test catalog from its actual recorded cells without creating payload bytes.
use super::*;
impl HistorySubsystem {
    pub(crate) fn replace_catalog_from_legacy_for_test(&self) {
        let (cells, existing) = {
            let ledger = self.ledger.read();
            (
                ledger
                    .commit_envelopes
                    .iter()
                    .map(|(id, cell)| (*id, Arc::clone(cell)))
                    .collect::<Vec<_>>(),
                ledger.commit_catalog.clone(),
            )
        };
        let mut catalog = RelationalCommitCatalog::default();
        let mut replacements = Vec::new();
        for (id, cell) in cells {
            let artifact = match existing.get(id) {
                Some(artifact) if Arc::ptr_eq(&artifact.envelope_residency(), &cell) => {
                    Arc::clone(artifact)
                }
                _ => {
                    let original = cell
                        .original_owner()
                        .expect("changed court cell retains its actual Original owner");
                    Arc::new(
                        crate::history::RelationalCommitArtifact::from_preencoded(original)
                            .expect("court envelope keeps canonical parentage"),
                    )
                }
            };
            replacements.push((id, artifact.envelope_residency()));
            catalog
                .append_preencoded_recovery(artifact)
                .expect("recorded cells have unique identities");
        }
        self.with_ledger_mut(|ledger| {
            ledger.commit_catalog = catalog;
            ledger.commit_envelopes = replacements.into_iter().collect();
        });
    }
}
