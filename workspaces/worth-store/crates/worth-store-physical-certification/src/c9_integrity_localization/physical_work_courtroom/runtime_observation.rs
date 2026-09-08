use serde_json::json;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalWorkRecoveryAdmissionOutcome,
    PhysicalWorkRecoveryIngressRejection, PhysicalWorkRecoveryObservationSubject,
};
use worth_store_physical_backend::FilesystemAccessPosture;

use super::process_protocol::{emit, Request};

pub(super) fn run(request: &Request) {
    let (serving, _) = super::open_store::open(
        &request.root,
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount),
    );
    let observations = serving.physical_recovery_admission_observations().iter().map(|observation| {
        let path = match observation.subject() {
            PhysicalWorkRecoveryObservationSubject::PendingFile(name) => format!("families/physical-work/{name}"),
            PhysicalWorkRecoveryObservationSubject::Inventory => "families/physical-work".to_owned(),
        };
        let (scope, outcome) = match observation.outcome() {
            PhysicalWorkRecoveryAdmissionOutcome::Admitted(scope) => (Some(scope), json!("Admitted")),
            PhysicalWorkRecoveryAdmissionOutcome::Rejected { scope, rejection } => {
                let outcome = match rejection {
                    PhysicalWorkRecoveryIngressRejection::Integrity(rejection) => serde_json::to_value(
                        super::super::process_integrity_projection::project_integrity_rejection(rejection)).unwrap(),
                    other => panic!("PW canonical row did not reach typed integrity: {other:?}"),
                };
                (scope, outcome)
            }
        };
        json!({"path": path, "scope": scope.map(super::super::process_integrity_projection::project_integrity_scope),
            "outcome": outcome})
    }).collect::<Vec<_>>();
    let counters = serving.physical_recovery_admission_counters();
    let dispositions = serving
        .physical_recovery_obligations()
        .iter()
        .map(|obligation| format!("{:?}", obligation.recovery_disposition()))
        .collect::<Vec<_>>();
    emit(
        request,
        json!({"store": super::super::process_execution::hex(serving.store_identity().bytes()),
            "observations": observations,
            "evidence_damaged": serving.physical_recovery_evidence_damaged(),
            "obligation_dispositions": dispositions,
            "counters": {"attempted": counters.attempted(), "admitted": counters.admitted_count(),
                "rejected": counters.rejected_before_owner_interpretation_count(),
                "owner_entries": counters.owner_interpretation_entries()},
        }),
    );
    // This subject scheduled no new work. The persisted obligation's own
    // disposition above (not this runtime's empty drain) carries inspection.
    serving.abort();
}
