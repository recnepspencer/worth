use crate::identity::hash_parts;

use super::super::fixtures::{
    legacy_delegation_parity_fixture, routing_delegation_parity_fixture,
    LegacyDelegationParityFixture, RoutingDelegationParityFixture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryIntentAdmissionLegacyParityLane {
    AuthoritativeExecution,
    EffectExecution,
    ReadExecutionCurrent,
    ReadExecutionInBasisContext,
    RoutingExecutionRuntime,
    RoutingExecutionWorkspace,
}

impl WorthQueryIntentAdmissionLegacyParityLane {
    fn as_str(self) -> &'static str {
        match self {
            Self::AuthoritativeExecution => "authoritative_execution",
            Self::EffectExecution => "effect_execution",
            Self::ReadExecutionCurrent => "read_execution_current",
            Self::ReadExecutionInBasisContext => "read_execution_in_basis_context",
            Self::RoutingExecutionRuntime => "routing_execution_runtime",
            Self::RoutingExecutionWorkspace => "routing_execution_workspace",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionLegacyParityCheck {
    label: String,
    passed: bool,
}

impl WorthQueryIntentAdmissionLegacyParityCheck {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn passed(&self) -> bool {
        self.passed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionLegacyParityRow {
    lane: WorthQueryIntentAdmissionLegacyParityLane,
    checks: Vec<WorthQueryIntentAdmissionLegacyParityCheck>,
    row_digest: String,
}

impl WorthQueryIntentAdmissionLegacyParityRow {
    pub fn lane(&self) -> WorthQueryIntentAdmissionLegacyParityLane {
        self.lane
    }

    pub fn checks(&self) -> &[WorthQueryIntentAdmissionLegacyParityCheck] {
        &self.checks
    }

    pub fn all_checks_pass(&self) -> bool {
        self.checks.iter().all(|check| check.passed())
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionLegacyParityReport {
    rows: Vec<WorthQueryIntentAdmissionLegacyParityRow>,
    legacy_delegation_parity_digest: String,
}

impl WorthQueryIntentAdmissionLegacyParityReport {
    pub fn rows(&self) -> &[WorthQueryIntentAdmissionLegacyParityRow] {
        &self.rows
    }

    pub fn legacy_delegation_parity_digest(&self) -> &str {
        &self.legacy_delegation_parity_digest
    }
}

pub fn worth_query_intent_admission_legacy_parity_report(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryIntentAdmissionLegacyParityReport {
    let fixture = legacy_delegation_parity_fixture(execution);
    let routing_fixture = routing_delegation_parity_fixture(execution);
    let rows = vec![
        authoritative_execution_row(&fixture),
        effect_execution_row(&fixture),
        read_execution_current_row(&fixture),
        read_execution_basis_context_row(&fixture),
        routing_execution_runtime_row(&routing_fixture),
        routing_execution_workspace_row(&routing_fixture),
    ];
    let legacy_delegation_parity_digest = hash_row_digests(&rows);
    WorthQueryIntentAdmissionLegacyParityReport {
        rows,
        legacy_delegation_parity_digest,
    }
}

fn authoritative_execution_row(
    fixture: &LegacyDelegationParityFixture,
) -> WorthQueryIntentAdmissionLegacyParityRow {
    parity_row(
        WorthQueryIntentAdmissionLegacyParityLane::AuthoritativeExecution,
        [
            digest_equality(
                "decision",
                fixture.authoritative_legacy.admission_decision_digest(),
                fixture.authoritative_canonical.admission_decision_digest(),
            ),
            digest_equality(
                "handoff",
                fixture.authoritative_legacy.execution_handoff_digest(),
                fixture.authoritative_canonical.execution_handoff_digest(),
            ),
            digest_equality(
                "binding",
                fixture.authoritative_legacy.execution_binding_digest(),
                fixture.authoritative_canonical.execution_binding_digest(),
            ),
            digest_equality(
                "provenance",
                fixture
                    .authoritative_legacy
                    .execution_provenance_chain_digest(),
                fixture
                    .authoritative_canonical
                    .execution_provenance_chain_digest(),
            ),
            digest_equality(
                "result",
                fixture.authoritative_legacy.receipt_digest(),
                fixture.authoritative_canonical.receipt_digest(),
            ),
        ],
    )
}

fn effect_execution_row(
    fixture: &LegacyDelegationParityFixture,
) -> WorthQueryIntentAdmissionLegacyParityRow {
    parity_row(
        WorthQueryIntentAdmissionLegacyParityLane::EffectExecution,
        [
            digest_equality(
                "decision",
                fixture
                    .effect_legacy
                    .intent_receipt()
                    .admission_decision_digest(),
                fixture
                    .effect_canonical
                    .intent_receipt()
                    .admission_decision_digest(),
            ),
            digest_equality(
                "handoff",
                fixture
                    .effect_legacy
                    .intent_receipt()
                    .execution_handoff_digest(),
                fixture
                    .effect_canonical
                    .intent_receipt()
                    .execution_handoff_digest(),
            ),
            digest_equality(
                "binding",
                fixture
                    .effect_legacy
                    .intent_receipt()
                    .execution_binding_digest(),
                fixture
                    .effect_canonical
                    .intent_receipt()
                    .execution_binding_digest(),
            ),
            digest_equality(
                "provenance",
                fixture.effect_legacy.execution_provenance_chain_digest(),
                fixture.effect_canonical.execution_provenance_chain_digest(),
            ),
            digest_equality(
                "result",
                fixture.effect_legacy.intent_receipt().receipt_digest(),
                fixture.effect_canonical.intent_receipt().receipt_digest(),
            ),
        ],
    )
}

fn read_execution_current_row(
    fixture: &LegacyDelegationParityFixture,
) -> WorthQueryIntentAdmissionLegacyParityRow {
    parity_row(
        WorthQueryIntentAdmissionLegacyParityLane::ReadExecutionCurrent,
        [
            optional_digest_equality(
                "trace",
                fixture
                    .read_current_legacy
                    .receipt()
                    .decision_trace_envelope()
                    .map(|trace| trace.trace_digest()),
                fixture
                    .read_current_canonical
                    .receipt()
                    .decision_trace_envelope()
                    .map(|trace| trace.trace_digest()),
            ),
            optional_digest_equality(
                "provenance",
                fixture
                    .read_current_legacy
                    .receipt()
                    .execution_provenance_chain_digest(),
                fixture
                    .read_current_canonical
                    .receipt()
                    .execution_provenance_chain_digest(),
            ),
            digest_equality(
                "result",
                fixture.read_current_legacy.receipt().result_digest(),
                fixture.read_current_canonical.receipt().result_digest(),
            ),
        ],
    )
}

fn read_execution_basis_context_row(
    fixture: &LegacyDelegationParityFixture,
) -> WorthQueryIntentAdmissionLegacyParityRow {
    parity_row(
        WorthQueryIntentAdmissionLegacyParityLane::ReadExecutionInBasisContext,
        [
            optional_digest_equality(
                "trace",
                fixture
                    .read_basis_legacy
                    .receipt()
                    .decision_trace_envelope()
                    .map(|trace| trace.trace_digest()),
                fixture
                    .read_basis_canonical
                    .receipt()
                    .decision_trace_envelope()
                    .map(|trace| trace.trace_digest()),
            ),
            optional_digest_equality(
                "provenance",
                fixture
                    .read_basis_legacy
                    .receipt()
                    .execution_provenance_chain_digest(),
                fixture
                    .read_basis_canonical
                    .receipt()
                    .execution_provenance_chain_digest(),
            ),
            digest_equality(
                "result",
                fixture.read_basis_legacy.receipt().result_digest(),
                fixture.read_basis_canonical.receipt().result_digest(),
            ),
        ],
    )
}

fn routing_execution_runtime_row(
    fixture: &RoutingDelegationParityFixture,
) -> WorthQueryIntentAdmissionLegacyParityRow {
    parity_row(
        WorthQueryIntentAdmissionLegacyParityLane::RoutingExecutionRuntime,
        [
            digest_equality(
                "trace",
                &fixture.runtime_legacy_trace_digest,
                fixture
                    .runtime_canonical
                    .receipt()
                    .decision_trace_envelope()
                    .expect("routing runtime canonical receipt should retain a trace")
                    .trace_digest(),
            ),
            digest_equality(
                "provenance",
                &fixture.runtime_legacy_provenance_digest,
                fixture
                    .runtime_canonical
                    .receipt()
                    .execution_provenance_chain_digest()
                    .expect("routing runtime canonical receipt should retain provenance"),
            ),
            digest_equality(
                "result",
                &fixture.runtime_legacy_probe_digest,
                fixture.runtime_canonical.receipt().probe_digest(),
            ),
        ],
    )
}

fn routing_execution_workspace_row(
    fixture: &RoutingDelegationParityFixture,
) -> WorthQueryIntentAdmissionLegacyParityRow {
    parity_row(
        WorthQueryIntentAdmissionLegacyParityLane::RoutingExecutionWorkspace,
        [
            digest_equality(
                "trace",
                &fixture.workspace_legacy_trace_digest,
                fixture
                    .workspace_canonical
                    .receipt()
                    .decision_trace_envelope()
                    .expect("routing workspace canonical receipt should retain a trace")
                    .trace_digest(),
            ),
            digest_equality(
                "provenance",
                &fixture.workspace_legacy_provenance_digest,
                fixture
                    .workspace_canonical
                    .receipt()
                    .execution_provenance_chain_digest()
                    .expect("routing workspace canonical receipt should retain provenance"),
            ),
            digest_equality(
                "result",
                &fixture.workspace_legacy_probe_digest,
                fixture.workspace_canonical.receipt().probe_digest(),
            ),
        ],
    )
}

fn parity_row<const N: usize>(
    lane: WorthQueryIntentAdmissionLegacyParityLane,
    checks: [String; N],
) -> WorthQueryIntentAdmissionLegacyParityRow {
    let checks = checks.into_iter().map(parse_check).collect::<Vec<_>>();
    WorthQueryIntentAdmissionLegacyParityRow {
        lane,
        row_digest: hash_parts(&[
            "worth_query_intent_admission_legacy_parity_row_v1".to_string(),
            format!("lane:{}", lane.as_str()),
            hash_parts(
                &checks
                    .iter()
                    .map(|check| format!("{}:{}", check.label(), check.passed()))
                    .collect::<Vec<_>>(),
            ),
        ]),
        checks,
    }
}

fn parse_check(check: String) -> WorthQueryIntentAdmissionLegacyParityCheck {
    let (label, passed) = check
        .split_once(':')
        .expect("legacy parity check should be label-prefixed");
    WorthQueryIntentAdmissionLegacyParityCheck {
        label: label.to_string(),
        passed: passed == "true",
    }
}

fn digest_equality(label: &str, left: &str, right: &str) -> String {
    format!("{label}:{}", left == right)
}

fn optional_digest_equality(label: &str, left: Option<&str>, right: Option<&str>) -> String {
    format!("{label}:{}", left == right)
}

fn hash_row_digests(rows: &[WorthQueryIntentAdmissionLegacyParityRow]) -> String {
    hash_parts(
        &rows
            .iter()
            .map(|row| row.row_digest().to_string())
            .collect::<Vec<_>>(),
    )
}
