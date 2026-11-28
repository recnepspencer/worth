const CLOSEOUT_PROOF_LANES = Object.freeze([
  "runtime",
  "typeSurface",
  "diagnosticsHistory",
  "performance",
]);

const CLOSEOUT_ROW_EVIDENCE = Object.freeze({
  localPatch: freezeCloseoutEvidence({
    runtimeTests: [
      "resource_effect_envelope.test.mjs",
      "resource_effect_branch_lifecycle.test.mjs",
    ],
    typeSurface: ["resource_effect_envelope_usage.ts"],
    diagnosticsHistory: ["phase5_history_closeout.test.mjs"],
    performance: ["json_item_aspect/cost_proof.test.mjs"],
  }),
  deliveryPatch: freezeCloseoutEvidence({
    runtimeTests: [
      "delivery_basis_history_closeout.test.mjs",
      "resource_effect_envelope.test.mjs",
    ],
    typeSurface: ["resource_effect_envelope_usage.ts"],
    diagnosticsHistory: ["delivery_basis_history_closeout.test.mjs"],
    performance: ["delivery_basis_history_closeout.test.mjs"],
  }),
  optimisticWrite: freezeCloseoutEvidence({
    runtimeTests: [
      "resource_effect_branch_lifecycle.test.mjs",
      "resource_effect_visible_selection.test.mjs",
    ],
    typeSurface: ["resource_api_effect_profiles_usage.ts"],
    diagnosticsHistory: ["resource_effect_visible_selection.test.mjs"],
    performance: ["resource_effect_branch_posture.test.mjs"],
  }),
  confirmation: freezeCloseoutEvidence({
    runtimeTests: ["resource_effect_server_confirmation.test.mjs"],
    typeSurface: ["resource_api_effect_profiles_usage.ts"],
    diagnosticsHistory: ["resource_effect_server_confirmation.test.mjs"],
    performance: ["full_resource_hostile_convergence.test.mjs"],
  }),
  failureRollback: freezeCloseoutEvidence({
    runtimeTests: ["resource_effect_rollback_execution.test.mjs"],
    typeSurface: ["resource_api_effect_profiles_usage.ts"],
    diagnosticsHistory: ["resource_effect_rollback_execution.test.mjs"],
    performance: ["resource_effect_rollback_execution.test.mjs"],
  }),
  branchRestore: freezeCloseoutEvidence({
    runtimeTests: [
      "branch_restore_action_surface.test.mjs",
      "full_resource_hostile_convergence.test.mjs",
    ],
    typeSurface: ["resource_effect_envelope_usage.ts"],
    diagnosticsHistory: ["branch_restore_action_surface.test.mjs"],
    performance: ["full_resource_hostile_convergence.test.mjs"],
  }),
  broadReplacement: freezeCloseoutEvidence({
    runtimeTests: [
      "response_lens/broad_response.test.mjs",
      "response_lens/topologies/detail_summary_response.test.mjs",
    ],
    typeSurface: ["resource_api_response_contract_usage.ts"],
    diagnosticsHistory: ["phase5_history_closeout.test.mjs"],
    performance: ["resource_response_topology_costs.ts"],
  }),
  diagnosticsHistory: freezeCloseoutEvidence({
    runtimeTests: [
      "full_resource_hostile_convergence.test.mjs",
      "phase5_history_closeout.test.mjs",
    ],
    typeSurface: ["resource_effect_envelope_usage.ts"],
    diagnosticsHistory: ["full_resource_hostile_convergence.test.mjs"],
    performance: ["resource_verification_package_helpers.mjs"],
  }),
});

function createResourceEffectCloseoutMatrix(profile) {
  return Object.freeze({
    profileName: profile.name,
    proofLanes: CLOSEOUT_PROOF_LANES,
    rows: Object.freeze([
      createCloseoutRow("localPatch", "admitted"),
      createCloseoutRow("deliveryPatch", "admitted"),
      createCloseoutRow(
        "optimisticWrite",
        profile.optimism === "branchSpeculative" ? "admitted" : "unsupported",
      ),
      createCloseoutRow("confirmation", profile.confirmation),
      createCloseoutRow("failureRollback", profile.rollback),
      createCloseoutRow(
        "branchRestore",
        profile.rollback === "unavailable" ? "unsupported" : profile.rollback,
      ),
      createCloseoutRow("broadReplacement", "admitted"),
      createCloseoutRow("diagnosticsHistory", "admitted"),
    ]),
  });
}

function createCloseoutRow(effectFamily, capability) {
  return Object.freeze({
    effectFamily,
    capability,
    runtimeProof: true,
    typeSurfaceProof: true,
    diagnosticsHistoryProof: true,
    performanceProof: true,
    evidence: CLOSEOUT_ROW_EVIDENCE[effectFamily],
  });
}

function freezeCloseoutEvidence(evidence) {
  return Object.freeze({
    runtimeTests: Object.freeze([...evidence.runtimeTests]),
    typeSurface: Object.freeze([...evidence.typeSurface]),
    diagnosticsHistory: Object.freeze([...evidence.diagnosticsHistory]),
    performance: Object.freeze([...evidence.performance]),
  });
}

export { createResourceEffectCloseoutMatrix };
