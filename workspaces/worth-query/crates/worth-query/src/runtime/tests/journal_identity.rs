use super::support::*;
use crate::application::{
    scan_journal_identity_forbidden_patterns, scan_journal_identity_required_pattern_failures,
    worth_query_journal_identity_inventory, WorthQueryJournalIdentityBoundaryPosture,
    WorthQueryJournalIdentityCertification, WorthQueryJournalIdentityInventoryEvidence,
    WorthQueryJournalIdentityOperationKind, WorthQueryJournalIdentityScheduleEvidence,
};

#[derive(Debug, Eq, PartialEq)]
struct JournalPositionSchedule {
    ordinals: Vec<u64>,
    evidence_identities: Vec<String>,
    positions: Vec<WorthQueryJournalPosition>,
}

impl JournalPositionSchedule {
    fn unique_identity_count(&self) -> usize {
        self.evidence_identities
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    fn position_schedule(&self) -> WorthQueryJournalPositionSchedule {
        WorthQueryJournalPositionSchedule::derive(self.positions.clone())
    }

    fn duplicate_first_position_schedule(&self) -> WorthQueryJournalPositionSchedule {
        let mut positions = self.positions.clone();
        positions.push(
            self.positions
                .first()
                .expect("schedule should carry at least one position")
                .clone(),
        );
        WorthQueryJournalPositionSchedule::derive(positions)
    }
}

fn submitted_batch_positions_with_inspection(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
) -> (JournalPositionSchedule, Vec<String>) {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace(workspace_name)
        .expect("task runtime should open a named workspace");
    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-1"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("First"),
                    )
                })
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-2"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Second"),
                    )
                })
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-3"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Third"),
                    )
                })
        })
        .expect("batch should commit");
    let inspection_identities = match workspace
        .inspect(request_execution, &receipt)
        .expect("batch receipt should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => inspection
            .journal_position_identities()
            .iter()
            .map(|identity| identity.as_str().to_string())
            .collect::<Vec<_>>(),
        _ => panic!("batch receipt should inspect as batch write receipt"),
    };
    let positions = receipt.journal_positions().collect::<Vec<_>>();

    assert!(positions
        .iter()
        .all(|position| position.authority() == WorthQueryJournalPositionAuthority::Committed));
    (
        JournalPositionSchedule {
            ordinals: positions
                .iter()
                .map(|position| position.ordinal_for_reporting())
                .collect(),
            evidence_identities: positions
                .iter()
                .map(|position| position.evidence_identity().as_str().to_string())
                .collect(),
            positions: positions.into_iter().cloned().collect(),
        },
        inspection_identities,
    )
}

fn submitted_schedule_positions(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
) -> JournalPositionSchedule {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace(workspace_name)
        .expect("task runtime should open a named workspace");
    let mut ordinals = Vec::new();
    let mut evidence_identities = Vec::new();
    let mut positions = Vec::new();
    for command in submission_schedule_commands() {
        let receipt = workspace
            .submissions()
            .expect("submission lane should be admitted")
            .submit(request_execution, command)
            .expect("scheduled submission should commit");
        assert_eq!(
            receipt.journal_position().authority(),
            WorthQueryJournalPositionAuthority::Committed
        );
        ordinals.push(receipt.journal_position().ordinal_for_reporting());
        evidence_identities.push(
            receipt
                .journal_position()
                .evidence_identity()
                .as_str()
                .to_string(),
        );
        positions.push(receipt.journal_position().clone());
    }
    JournalPositionSchedule {
        ordinals,
        evidence_identities,
        positions,
    }
}

fn submission_schedule_commands() -> Vec<WorthQueryWriteCommand> {
    ["First", "Second", "Third"]
        .iter()
        .enumerate()
        .map(|(index, title)| {
            WorthQueryAspectMutationBuilder::new()
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value(format!("task-{}", index + 1)),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value(*title),
                )
                .build_insert("Task")
                .expect("schedule insert command should build")
        })
        .collect()
}

fn journal_identity_inventory_evidence() -> WorthQueryJournalIdentityInventoryEvidence {
    let workspace_root = workspace_root();
    let forbidden_count = scan_journal_identity_forbidden_patterns(&workspace_root).len();
    let required_count = scan_journal_identity_required_pattern_failures(&workspace_root).len();
    let operations = worth_query_journal_identity_inventory()
        .iter()
        .map(|row| row.kind())
        .collect::<std::collections::BTreeSet<_>>();
    let missing_count = missing_operation_count(&operations);
    WorthQueryJournalIdentityInventoryEvidence::new(
        forbidden_count,
        required_count,
        missing_count,
        inventory_digest(forbidden_count, required_count, missing_count),
    )
}

fn missing_operation_count(
    operations: &std::collections::BTreeSet<WorthQueryJournalIdentityOperationKind>,
) -> usize {
    required_operations()
        .iter()
        .filter(|operation| !operations.contains(operation))
        .count()
}

fn required_operations() -> &'static [WorthQueryJournalIdentityOperationKind] {
    &[
        WorthQueryJournalIdentityOperationKind::MintCommittedPosition,
        WorthQueryJournalIdentityOperationKind::MintPreviewPosition,
        WorthQueryJournalIdentityOperationKind::AdmitSubmissionLanePosition,
        WorthQueryJournalIdentityOperationKind::CarryWriteReceiptPosition,
        WorthQueryJournalIdentityOperationKind::ReadWriteReceiptPosition,
        WorthQueryJournalIdentityOperationKind::CarryPreviewReceiptPosition,
        WorthQueryJournalIdentityOperationKind::CarryBatchReceiptPositions,
        WorthQueryJournalIdentityOperationKind::BuildJournalSegmentIdentity,
        WorthQueryJournalIdentityOperationKind::BuildJournalReplayRequest,
        WorthQueryJournalIdentityOperationKind::RetainReplayRegistryReceipt,
        WorthQueryJournalIdentityOperationKind::MaterializeReplayOutcome,
        WorthQueryJournalIdentityOperationKind::ExposeWorkspaceReplayFacade,
        WorthQueryJournalIdentityOperationKind::CertifyJournalIdentityBoundary,
    ]
}

fn inventory_digest(forbidden_count: usize, required_count: usize, missing_count: usize) -> String {
    crate::evidence_identity::worth_query_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceScope::ApplicationSupportReport,
    )
    .field_usize(
        crate::evidence_identity::WorthQueryEvidenceTag::new("forbidden_count"),
        forbidden_count,
    )
    .field_usize(
        crate::evidence_identity::WorthQueryEvidenceTag::new("required_count"),
        required_count,
    )
    .field_usize(
        crate::evidence_identity::WorthQueryEvidenceTag::new("missing_count"),
        missing_count,
    )
    .seal()
    .as_str()
    .to_string()
}

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}

mod inventory;
mod submitted_positions;
