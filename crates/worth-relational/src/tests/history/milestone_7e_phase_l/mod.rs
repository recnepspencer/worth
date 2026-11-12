mod fixtures;

use crate::facade::inspection::{
    RelationalMergeSupportInspectionAbsenceKind, RelationalMergeSupportInspectionRow,
};
use crate::inspection::data::RelationalMergeSupportInspectionInput;
use crate::inspection::support_inspection_witness;

use fixtures::{prepared_merge, runtime_with_schema_declared_entity_policy};

#[test]
fn merge_support_inspection_witness_denies_missing_required_proof_family() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_schema_declared_entity_policy();
    let prepared = prepared_merge(execution, &runtime);
    let correspondence = runtime
        .merge()
        .retain_merge_correspondence_witness_from_prepared_execution(&prepared);
    let input = RelationalMergeSupportInspectionInput {
        request: prepared.request().clone(),
        branch_basis: prepared.execution_ready_plan().basis.clone(),
        proof_packet: None,
        correspondence_witness: Some(correspondence.clone()),
        schema_reconciliation_witness: Some(
            prepared.artifact().schema_reconciliation_witness.clone(),
        ),
        strategy_witness: Some(prepared.artifact().strategy_witness.clone()),
    };
    let missing_packet = support_inspection_witness(input.clone()).expect("missing packet support");
    let missing_correspondence =
        support_inspection_witness(RelationalMergeSupportInspectionInput {
            correspondence_witness: None,
            ..input.clone()
        })
        .expect("missing correspondence support");
    let missing_schema = support_inspection_witness(RelationalMergeSupportInspectionInput {
        schema_reconciliation_witness: None,
        ..input.clone()
    })
    .expect("missing schema support");
    let missing_strategy = support_inspection_witness(RelationalMergeSupportInspectionInput {
        strategy_witness: None,
        ..input
    })
    .expect("missing strategy support");

    assert!(missing_packet.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::RequestAdmission {
            absence: Some(RelationalMergeSupportInspectionAbsenceKind::MissingProofPacket),
            ..
        }
    )));
    assert!(missing_correspondence.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Correspondence {
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingCorrespondenceWitness
            ),
            ..
        }
    )));
    assert!(missing_schema.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Schema {
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingSchemaReconciliationWitness
            ),
            ..
        }
    )));
    assert!(missing_strategy.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Strategy {
            absence: Some(RelationalMergeSupportInspectionAbsenceKind::MissingStrategyWitness),
            ..
        }
    )));
}
