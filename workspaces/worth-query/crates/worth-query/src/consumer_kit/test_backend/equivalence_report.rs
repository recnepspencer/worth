use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryAspectTouch, WorthQueryWriteReceipt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTestBackendEquivalenceRow {
    field: &'static str,
    matched: bool,
    expected: String,
    found: String,
}

impl WorthQueryTestBackendEquivalenceRow {
    fn present(field: &'static str, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            field,
            matched: true,
            expected: value.clone(),
            found: value,
        }
    }

    fn compare(field: &'static str, expected: impl Into<String>, found: impl Into<String>) -> Self {
        let expected = expected.into();
        let found = found.into();
        Self {
            field,
            matched: expected == found,
            expected,
            found,
        }
    }

    pub fn field(&self) -> &'static str {
        self.field
    }

    pub fn matched(&self) -> bool {
        self.matched
    }

    pub fn expected(&self) -> &str {
        &self.expected
    }

    pub fn found(&self) -> &str {
        &self.found
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTestBackendEquivalenceReport {
    rows: Vec<WorthQueryTestBackendEquivalenceRow>,
    report_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryTestBackendEquivalenceReport {
    fn new(
        rows: Vec<WorthQueryTestBackendEquivalenceRow>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let report_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerEvidenceReport)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "in-memory-test-backend-equivalence",
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("field"),
                    rows.iter().map(WorthQueryTestBackendEquivalenceRow::field),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("matched"),
                    rows.iter()
                        .map(|row| if row.matched() { "matched" } else { "mismatch" }),
                )
                .seal()?;
        Ok(Self {
            rows,
            report_identity,
        })
    }

    pub fn rows(&self) -> &[WorthQueryTestBackendEquivalenceRow] {
        &self.rows
    }

    pub fn matched(&self) -> bool {
        self.rows
            .iter()
            .all(WorthQueryTestBackendEquivalenceRow::matched)
    }

    pub fn report_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.report_identity
    }
}

pub fn compare_test_backend_write_receipts(
    in_memory: &WorthQueryWriteReceipt,
    bridge: &WorthQueryWriteReceipt,
) -> Result<WorthQueryTestBackendEquivalenceReport, IdentityDerivationDenial> {
    WorthQueryTestBackendEquivalenceReport::new(vec![
        WorthQueryTestBackendEquivalenceRow::compare(
            "mutation_family",
            in_memory.mutation_family().as_str(),
            bridge.mutation_family().as_str(),
        ),
        WorthQueryTestBackendEquivalenceRow::compare(
            "authority_lane",
            format!("{:?}", in_memory.authority_lane()),
            format!("{:?}", bridge.authority_lane()),
        ),
        WorthQueryTestBackendEquivalenceRow::compare(
            "semantic_deltas",
            semantic_delta_rows(in_memory),
            semantic_delta_rows(bridge),
        ),
        WorthQueryTestBackendEquivalenceRow::compare(
            "affected_live_view_count",
            in_memory.affected_live_view_targets().len().to_string(),
            bridge.affected_live_view_targets().len().to_string(),
        ),
        WorthQueryTestBackendEquivalenceRow::present(
            "in_memory_commit_identity_present",
            (!in_memory.commit_identity().is_empty()).to_string(),
        ),
        WorthQueryTestBackendEquivalenceRow::present(
            "bridge_commit_identity_present",
            (!bridge.commit_identity().is_empty()).to_string(),
        ),
        WorthQueryTestBackendEquivalenceRow::present(
            "in_memory_snapshot_identity_present",
            (!in_memory
                .snapshot_evidence_identity()
                .terminal_projection_for_reporting()
                .is_empty())
            .to_string(),
        ),
        WorthQueryTestBackendEquivalenceRow::present(
            "bridge_snapshot_identity_present",
            (!bridge
                .snapshot_evidence_identity()
                .terminal_projection_for_reporting()
                .is_empty())
            .to_string(),
        ),
    ])
}

fn semantic_delta_rows(receipt: &WorthQueryWriteReceipt) -> String {
    receipt
        .deltas()
        .iter()
        .map(|delta| {
            format!(
                "{}:{:?}:{}",
                delta.collection(),
                delta.kind(),
                terminal_aspect_touch_digest_parts(delta.admitted_touched_aspects())
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn terminal_aspect_touch_digest_parts(touches: &[WorthQueryAspectTouch]) -> Vec<String> {
    touches
        .iter()
        .map(WorthQueryAspectTouch::admitted_touch_digest_part)
        .collect()
}
