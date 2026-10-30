use crate::admission::{
    UiAdmissionTarget, UiAdmissionWorld, UiMeasurementAdmission, UiMeasurementAdmissionPosture,
    UiMeasurementCapabilityGateReason, UiMeasurementUnsupportedReason, UiSupportPosture,
    UiSupportSnapshot,
};
use crate::evidence::UiEvidenceAuthorityGeneration;
use crate::obligations::catalog::UiObligationFamily;
use crate::obligations::selection::UiSelectedObligationSet;

use super::UiAdmissionBoundary;

#[path = "measurement_entry_support.rs"]
mod measurement_entry_support;

use measurement_entry_support::{
    measurement_support_snapshot_from_admission, selection_target_for_touch,
    support_posture_for_measurement_obligation, touch_has_measurement_lane,
};

impl<'a> UiAdmissionBoundary<'a> {
    pub fn select_obligations(
        &self,
        touch: &crate::obligations::touch::UiGraphTouchDescriptor,
    ) -> UiSelectedObligationSet {
        self.select_obligations_for_target(
            touch,
            UiAdmissionTarget::graph_node(
                touch.target().graph_node_identity(),
                UiAdmissionWorld::from_graph_world_profile(touch.world().world_profile().clone()),
            ),
        )
    }

    pub fn select_obligations_for_target(
        &self,
        touch: &crate::obligations::touch::UiGraphTouchDescriptor,
        target: UiAdmissionTarget,
    ) -> UiSelectedObligationSet {
        let selection_target = selection_target_for_touch(touch, &target);
        let support_snapshot = self.support_snapshot(&selection_target);

        crate::obligations::selection::UiObligationSelectionBoundary::new(
            self.support_artifacts,
            self.graph_snapshot,
        )
        .select(touch, support_snapshot, target)
    }

    pub fn lower_obligation_dispatch(
        &self,
        selected: &UiSelectedObligationSet,
    ) -> crate::obligations::dispatch::UiObligationDispatchPlan {
        let measurement_admission = self.admit_measurement_requirement(selected);
        crate::obligations::dispatch::UiObligationDispatchBoundary::new().lower(
            selected,
            self.effective_support_snapshot(selected, measurement_admission.clone()),
            measurement_admission,
        )
    }

    pub fn dispatch_selected_obligations(
        &self,
        selected: &UiSelectedObligationSet,
    ) -> Box<[crate::obligations::verdict::UiObligationVerdict]> {
        self.lower_obligation_dispatch(selected).execute()
    }

    pub fn admit_selected_obligations(
        &self,
        selected: &UiSelectedObligationSet,
    ) -> crate::admission::UiAdmissionReport {
        let measurement_admission = self.admit_measurement_requirement(selected);
        let support_snapshot =
            self.effective_support_snapshot(selected, measurement_admission.clone());
        let dispatch_plan = crate::obligations::dispatch::UiObligationDispatchBoundary::new()
            .lower(
                selected,
                support_snapshot.clone(),
                measurement_admission.clone(),
            );
        let verdicts = dispatch_plan.execute();
        crate::admission::UiAdmissionReport::from_selected_execution(
            selected,
            support_snapshot,
            measurement_admission,
            dispatch_plan,
            verdicts,
        )
    }

    fn selected_measurement_support_snapshot(
        &self,
        selected: &UiSelectedObligationSet,
    ) -> Option<UiSupportSnapshot> {
        let measurement_obligation =
            selected.obligation_for_family(UiObligationFamily::MeasurementRequirement)?;
        Some(UiSupportSnapshot::new(
            selected.requested_target().clone(),
            support_posture_for_measurement_obligation(
                selected.requested_target(),
                measurement_obligation.support_posture(),
                selected.touch().world().world_profile(),
            ),
        ))
    }

    pub fn admit_measurement_requirement(
        &self,
        selected: &UiSelectedObligationSet,
    ) -> Option<UiMeasurementAdmission> {
        if !touch_has_measurement_lane(selected.touch()) {
            return None;
        }

        let target = selected.requested_target().clone();
        let boundary_generation =
            UiEvidenceAuthorityGeneration::new(self.graph_snapshot.generation().as_u64());
        let measurement_obligation =
            selected.obligation_for_family(UiObligationFamily::MeasurementRequirement);
        let declaration_identity = self
            .graph_node_record(&target)
            .map(|node_record| node_record.declaration_identity().clone())
            .or_else(|| selected.selected_declaration_identity().cloned());
        let host_capability_profile_digest = target
            .host_capability_report()
            .map(worth_ui_host_contract::WorthUiHostCapabilityReport::profile_identity_digest);
        let host_capability_observation_generation = target
            .host_capability_report()
            .map(worth_ui_host_contract::WorthUiHostCapabilityReport::observation_generation);

        let admission = |posture| {
            UiMeasurementAdmission::new(crate::admission::UiMeasurementAdmissionInput {
                target: target.clone(),
                graph_node_identity: target.graph_node_identity(),
                declaration_identity: declaration_identity.clone(),
                touch_identity_digest: selected.touch().identity_digest(),
                selected_measurement_obligation_identity_digest: measurement_obligation
                    .map(|obligation| obligation.identity().identity_digest()),
                selected_support_authority_generation: selected.authority_generation(),
                boundary_support_authority_generation: boundary_generation,
                host_capability_profile_digest,
                host_capability_observation_generation,
                posture,
            })
        };

        if target.graph_node_identity() != selected.touch().target().graph_node_identity() {
            return Some(admission(
                UiMeasurementAdmissionPosture::StaleSupportPosture {
                    world: target.world().clone(),
                    selected_generation: selected.authority_generation(),
                    boundary_generation,
                },
            ));
        }

        if target.world().graph_world_profile() != self.graph_snapshot.world_profile() {
            return Some(admission(UiMeasurementAdmissionPosture::WrongWorld {
                expected: UiAdmissionWorld::from_graph_world_profile(
                    self.graph_snapshot.world_profile().clone(),
                ),
                observed: target.world().clone(),
            }));
        }

        let Some(node_record) = self.graph_node_record(&target) else {
            return Some(admission(
                UiMeasurementAdmissionPosture::StaleSupportPosture {
                    world: target.world().clone(),
                    selected_generation: selected.authority_generation(),
                    boundary_generation,
                },
            ));
        };

        if selected.selected_declaration_identity() != Some(node_record.declaration_identity()) {
            return Some(admission(
                UiMeasurementAdmissionPosture::StaleSupportPosture {
                    world: target.world().clone(),
                    selected_generation: selected.authority_generation(),
                    boundary_generation,
                },
            ));
        }

        let Some(artifact) = self.support_artifact(node_record.declaration_identity()) else {
            return Some(admission(
                UiMeasurementAdmissionPosture::StaleSupportPosture {
                    world: target.world().clone(),
                    selected_generation: selected.authority_generation(),
                    boundary_generation,
                },
            ));
        };

        let Ok(snapshot) = artifact.support_snapshot() else {
            return Some(admission(
                UiMeasurementAdmissionPosture::StaleSupportPosture {
                    world: target.world().clone(),
                    selected_generation: selected.authority_generation(),
                    boundary_generation,
                },
            ));
        };

        let Some(_row) =
            snapshot.row(crate::declaration::UiDeclarationSupportRowSchemaKind::TouchMeaning)
        else {
            return Some(admission(
                UiMeasurementAdmissionPosture::StaleSupportPosture {
                    world: target.world().clone(),
                    selected_generation: selected.authority_generation(),
                    boundary_generation,
                },
            ));
        };

        let fresh_touch_support = self.support_snapshot(&UiAdmissionTarget::graph_node(
            target.graph_node_identity(),
            target.world().clone(),
        ));
        if fresh_touch_support.posture() != selected.support_snapshot().posture()
            || selected.authority_generation() != boundary_generation
        {
            return Some(admission(
                UiMeasurementAdmissionPosture::StaleSupportPosture {
                    world: target.world().clone(),
                    selected_generation: selected.authority_generation(),
                    boundary_generation,
                },
            ));
        }

        if let Some(measurement_support_snapshot) =
            self.selected_measurement_support_snapshot(selected)
        {
            match measurement_support_snapshot.posture() {
                UiSupportPosture::Supported { .. } => {
                    let Some(host_capability_report) = target.host_capability_report() else {
                        return Some(admission(UiMeasurementAdmissionPosture::CapabilityGated {
                            world: target.world().clone(),
                            reason: UiMeasurementCapabilityGateReason::MissingHostCapabilityReport,
                        }));
                    };
                    let posture = match host_capability_report.posture() {
                        worth_ui_host_contract::WorthUiHostCapabilityPosture::Available => {
                            UiMeasurementAdmissionPosture::Admitted {
                                world: target.world().clone(),
                                host_capability: host_capability_report.clone(),
                            }
                        }
                        worth_ui_host_contract::WorthUiHostCapabilityPosture::Missing => {
                            UiMeasurementAdmissionPosture::CapabilityGated {
                                world: target.world().clone(),
                                reason: UiMeasurementCapabilityGateReason::MissingHostCapability,
                            }
                        }
                        worth_ui_host_contract::WorthUiHostCapabilityPosture::Ambiguous => {
                            UiMeasurementAdmissionPosture::CapabilityGated {
                                world: target.world().clone(),
                                reason: UiMeasurementCapabilityGateReason::AmbiguousHostCapability,
                            }
                        }
                        worth_ui_host_contract::WorthUiHostCapabilityPosture::DiagnosticOnly => {
                            UiMeasurementAdmissionPosture::CapabilityGated {
                                world: target.world().clone(),
                                reason:
                                    UiMeasurementCapabilityGateReason::DiagnosticOnlyHostCapability,
                            }
                        }
                    };
                    return Some(admission(posture));
                }
                UiSupportPosture::Unsupported { reason, .. } => {
                    return Some(admission(UiMeasurementAdmissionPosture::Unsupported {
                        world: target.world().clone(),
                        reason: UiMeasurementUnsupportedReason::Support(*reason),
                    }));
                }
                UiSupportPosture::Deferred { expected_in, .. } => {
                    return Some(admission(UiMeasurementAdmissionPosture::Deferred {
                        world: target.world().clone(),
                        expected_in: *expected_in,
                    }));
                }
                UiSupportPosture::DiagnosticOnly { .. } => {
                    return Some(admission(UiMeasurementAdmissionPosture::DiagnosticOnly {
                        world: target.world().clone(),
                    }));
                }
                UiSupportPosture::WrongWorld {
                    expected, observed, ..
                } => {
                    return Some(admission(UiMeasurementAdmissionPosture::WrongWorld {
                        expected: expected.clone(),
                        observed: observed.clone(),
                    }));
                }
            }
        }

        Some(admission(UiMeasurementAdmissionPosture::Unsupported {
            world: target.world().clone(),
            reason: UiMeasurementUnsupportedReason::SelectionDidNotYieldMeasurementRequirement,
        }))
    }

    fn effective_support_snapshot(
        &self,
        selected: &UiSelectedObligationSet,
        measurement_admission: Option<UiMeasurementAdmission>,
    ) -> UiSupportSnapshot {
        measurement_support_snapshot_from_admission(measurement_admission)
            .unwrap_or_else(|| selected.support_snapshot().clone())
    }
}
