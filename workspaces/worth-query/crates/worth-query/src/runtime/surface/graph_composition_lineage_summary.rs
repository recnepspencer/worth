use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryContinuityMutationEvidence, WorthQueryContinuityMutationFamily,
    WorthQueryContinuityOutcomeClass, WorthQueryMutationAuthorityIdentity,
    WorthQueryMutationEvidenceDigest, WorthQueryMutationTargetCollectionIdentity,
    WorthQueryWriteReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionLineageEntry {
    component_index: usize,
    family: WorthQueryContinuityMutationFamily,
    outcome_class: WorthQueryContinuityOutcomeClass,
    prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    successor_authoritative_identities: Vec<WorthQueryMutationAuthorityIdentity>,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    lineage_digest: WorthQueryMutationEvidenceDigest,
    continuity_resolution_digest: WorthQueryMutationEvidenceDigest,
}

impl WorthQueryGraphCompositionLineageEntry {
    fn new(
        component_index: usize,
        receipt: &WorthQueryWriteReceipt,
        evidence: &WorthQueryContinuityMutationEvidence,
    ) -> Self {
        Self {
            component_index,
            family: evidence.family(),
            outcome_class: evidence.outcome_class(),
            prior_authoritative_identity: evidence.prior_authoritative_identity().clone(),
            successor_authoritative_identities: evidence
                .successor_authoritative_identities()
                .to_vec(),
            target_collection: receipt
                .target_collection_identity()
                .cloned()
                .or_else(|| receipt.declared_collection_identity().cloned())
                .or_else(|| evidence.target_collection().cloned()),
            lineage_digest: evidence.lineage_digest().clone(),
            continuity_resolution_digest: evidence.continuity_resolution_digest().clone(),
        }
    }

    pub fn component_index(&self) -> usize {
        self.component_index
    }

    pub fn family(&self) -> WorthQueryContinuityMutationFamily {
        self.family
    }

    pub fn outcome_class(&self) -> WorthQueryContinuityOutcomeClass {
        self.outcome_class
    }

    pub fn prior_authoritative_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.prior_authoritative_identity
    }

    pub fn successor_authoritative_identities(&self) -> &[WorthQueryMutationAuthorityIdentity] {
        &self.successor_authoritative_identities
    }

    pub fn target_collection(&self) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }

    pub fn lineage_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.lineage_digest
    }

    pub fn continuity_resolution_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.continuity_resolution_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionLineageSummary {
    entries: Vec<WorthQueryGraphCompositionLineageEntry>,
    counter_snapshot: String,
    aggregate_lineage_digest: WorthQueryEvidenceIdentity,
    aggregate_continuity_resolution_digest: WorthQueryEvidenceIdentity,
    lineage_summary_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphCompositionLineageSummary {
    pub(in crate::runtime) fn derive(
        write_receipts: &[WorthQueryWriteReceipt],
    ) -> Result<Option<Self>, IdentityDerivationDenial> {
        let entries = write_receipts
            .iter()
            .enumerate()
            .filter_map(|(component_index, receipt)| {
                receipt.continuity_mutation_evidence().map(|evidence| {
                    WorthQueryGraphCompositionLineageEntry::new(component_index, receipt, evidence)
                })
            })
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return Ok(None);
        }

        let single_successor_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_class()
                    == WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
            })
            .count();
        let split_successor_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_class()
                    == WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
            })
            .count();
        let merge_successor_count = entries
            .iter()
            .filter(|entry| {
                entry.outcome_class()
                    == WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
            })
            .count();
        let rejected_count =
            entries.len() - single_successor_count - split_successor_count - merge_successor_count;
        let counter_snapshot = diagnostic_counter_snapshot(&[
            ("continuity_entries", entries.len()),
            ("single_successors", single_successor_count),
            ("split_successors", split_successor_count),
            ("merge_successors", merge_successor_count),
            ("rejections", rejected_count),
        ]);
        let aggregate_lineage_digest = aggregate_digest(
            "worth_query_graph_composition_lineage_digest_v1",
            entries
                .iter()
                .map(|entry| entry.lineage_digest().evidence_identity()),
        )?;
        let aggregate_continuity_resolution_digest = aggregate_digest(
            "worth_query_graph_composition_lineage_resolution_digest_v1",
            entries
                .iter()
                .map(|entry| entry.continuity_resolution_digest().evidence_identity()),
        )?;
        let entry_digests = entries
            .iter()
            .map(graph_composition_lineage_entry_digest)
            .collect::<Result<Vec<_>, _>>()?;
        let aggregate_entry_digest = aggregate_digest(
            "worth_query_graph_composition_lineage_entry_digest_v1",
            entry_digests.iter(),
        )?;
        let lineage_summary_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-lineage-summary",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("lineage"),
                    &aggregate_lineage_digest,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("resolution"),
                    &aggregate_continuity_resolution_digest,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("continuity_entry_count"),
                    entries.len(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("single_successor_count"),
                    single_successor_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("split_successor_count"),
                    split_successor_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("merge_successor_count"),
                    merge_successor_count,
                )
                .field_usize(WorthQueryEvidenceTag::new("rejected_count"), rejected_count)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("entries"),
                    &aggregate_entry_digest,
                )
                .seal()?;

        Ok(Some(Self {
            entries,
            counter_snapshot,
            aggregate_lineage_digest,
            aggregate_continuity_resolution_digest,
            lineage_summary_digest,
        }))
    }

    pub fn entries(&self) -> &[WorthQueryGraphCompositionLineageEntry] {
        &self.entries
    }

    pub fn counter_snapshot(&self) -> &str {
        &self.counter_snapshot
    }

    pub fn aggregate_lineage_digest(&self) -> &str {
        self.aggregate_lineage_digest.as_str()
    }

    pub fn aggregate_continuity_resolution_digest(&self) -> &str {
        self.aggregate_continuity_resolution_digest.as_str()
    }

    pub fn lineage_summary_digest(&self) -> &str {
        self.lineage_summary_digest.as_str()
    }

    pub fn lineage_summary_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.lineage_summary_digest
    }
}

fn diagnostic_counter_snapshot(fields: &[(&str, usize)]) -> String {
    let mut snapshot = String::new();
    for (index, (label, value)) in fields.iter().enumerate() {
        if index > 0 {
            snapshot.push(';');
        }
        snapshot.push_str(label);
        snapshot.push('=');
        snapshot.push_str(&value.to_string());
    }
    snapshot
}

fn aggregate_digest<'a>(
    label: &str,
    digests: impl IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), label)
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("digest"), digests)
            .seal()?,
    )
}

fn graph_composition_lineage_entry_digest(
    entry: &WorthQueryGraphCompositionLineageEntry,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "graph-composition-lineage-entry",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("component"),
                entry.component_index(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                entry.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("outcome"),
                entry.outcome_class().as_str(),
            );
    if let Some(collection) = entry.target_collection() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("target_collection"),
            collection.evidence_identity(),
        );
    }
    identity = identity
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("prior_authority"),
            entry.prior_authoritative_identity().evidence_identity(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("successor_authority"),
            entry
                .successor_authoritative_identities()
                .iter()
                .map(|identity| identity.evidence_identity()),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("lineage"),
            entry.lineage_digest().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("continuity_resolution"),
            entry.continuity_resolution_digest().evidence_identity(),
        );
    Ok(identity.seal()?)
}
