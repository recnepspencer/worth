use super::super::super::super::support::*;

#[test]
fn runtime_public_read_composition_phase_gate_freezes_completion_and_blockers() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.read-composition-phase-gate")
        .expect("task runtime should open a named workspace");
    let support = workspace.public_read_composition_support_report();
    let closeout = workspace.public_read_composition_phase_one_closeout();
    let gate = workspace.public_read_composition_phase_gate();

    assert_eq!(
        gate.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(gate.read_support_digest(), support.support_digest());
    assert_eq!(gate.phase_one_closeout_digest(), closeout.closeout_digest());
    assert_eq!(
        gate.support_matrix_digest(),
        workspace.public_support_matrix().matrix_digest().as_str()
    );
    assert_eq!(gate.phase_two_start_family(), "loop_cycle_neighborhood");
    assert_gate_row(
        &gate,
        WorthQueryReadCompositionPhaseGateFamily::PhaseOneKernelComplete,
        WorthQueryReadCompositionPhaseGateStatus::Satisfied,
        "generic read kernel is frozen",
    );
    assert_gate_row(
        &gate,
        WorthQueryReadCompositionPhaseGateFamily::PhaseTwoWorthAdoptionReady,
        WorthQueryReadCompositionPhaseGateStatus::Satisfied,
        "Worth may begin domain adoption",
    );
    assert_gate_row(
        &gate,
        WorthQueryReadCompositionPhaseGateFamily::PhaseThreeAggregateProofComplete,
        WorthQueryReadCompositionPhaseGateStatus::Satisfied,
        "Worth topology now exposes aggregate",
    );
    assert_gate_row(
        &gate,
        WorthQueryReadCompositionPhaseGateFamily::MilestoneThreeResumeReady,
        WorthQueryReadCompositionPhaseGateStatus::Satisfied,
        "Milestone 3 may resume",
    );
    assert!(!gate.gate_digest().is_empty());
}

fn assert_gate_row(
    gate: &WorthQueryReadCompositionPhaseGate,
    family: WorthQueryReadCompositionPhaseGateFamily,
    status: WorthQueryReadCompositionPhaseGateStatus,
    reason_fragment: &str,
) {
    assert!(gate.rows().iter().any(|row| {
        row.family() == family && row.status() == status && row.reason().contains(reason_fragment)
    }));
}
