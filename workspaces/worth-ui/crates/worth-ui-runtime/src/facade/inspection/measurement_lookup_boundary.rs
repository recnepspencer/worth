use worth_ui_inspection::{
    UiEvidenceAuthorityGeneration, UiInspectionMeasurementDenialPosture,
    UiInspectionMeasurementEvidenceSlot, UiInspectionMeasurementEvidenceView,
    UiInspectionMeasurementFailureSource, UiInspectionQuery, UiInspectionScope,
    UiInspectionSupportReport, UiInspectionTarget,
};

use crate::admission::UiMeasurementAdmissionPosture;
use crate::declaration::{UiDeclarationArtifact, UiDeclarationAuthoredEvidenceIndex};
use crate::evidence::{
    admit_measurement_basis, project_measurement_inspection_compatibility_view,
    project_measurement_inspection_denial_view, project_measurement_inspection_view,
    MeasurementEvidenceInput, UiEvidenceMaterializedDetail, UiEvidenceSliceAssembly,
    UiEvidenceSliceAssemblyInput, UiInspectionCostMetrics,
};
use crate::facade::inspection_bridge::UiInspectionReceipt;
use crate::graph::{UiGraphNodeEvidenceIndex, UiGraphSnapshot};

use super::measurement_lookup_support::{
    admission_target_for_touch, declaration_instance_resolution, filter_refs_for_query,
    measurement_bundle_for_artifact, measurement_host_inputs, measurement_policy_for_artifact,
    measurement_touch_for_target, query_measurement_outcome_for_bundle,
    support_report_for_artifact, MeasurementInspectionOutcome, MeasurementInspectionTarget,
    QueryMeasurementInspectionOutcome,
};

pub(crate) struct WorthUiMeasurementInspectionBoundary<'a> {
    declaration_artifacts: &'a [UiDeclarationArtifact],
    graph_snapshot: &'a UiGraphSnapshot,
    authored_evidence_index: &'a UiDeclarationAuthoredEvidenceIndex,
    graph_node_evidence_index: &'a UiGraphNodeEvidenceIndex,
    measurement_evidence:
        &'a crate::facade::measurement_inspection_evidence::UiMeasurementInspectionEvidenceSnapshot,
}

impl<'a> WorthUiMeasurementInspectionBoundary<'a> {
    pub(crate) const fn new(
        declaration_artifacts: &'a [UiDeclarationArtifact],
        graph_snapshot: &'a UiGraphSnapshot,
        authored_evidence_index: &'a UiDeclarationAuthoredEvidenceIndex,
        graph_node_evidence_index: &'a UiGraphNodeEvidenceIndex,
        measurement_evidence: &'a crate::facade::measurement_inspection_evidence::UiMeasurementInspectionEvidenceSnapshot,
    ) -> Self {
        Self {
            declaration_artifacts,
            graph_snapshot,
            authored_evidence_index,
            graph_node_evidence_index,
            measurement_evidence,
        }
    }

    pub(crate) fn inspect(
        &self,
        app: &crate::facade::WorthUiApp,
        query: UiInspectionQuery,
        authority_generation: UiEvidenceAuthorityGeneration,
    ) -> Option<UiInspectionReceipt> {
        if query.scope() != UiInspectionScope::Measurement {
            return None;
        }

        let target = self.lookup_target(query.target())?;
        let support_report = support_report_for_artifact(target.artifact, query.scope())?;
        let detail = UiEvidenceMaterializedDetail::Measurement(self.measurement_view_for_target(
            app,
            &target,
            support_report,
        ));
        let relevance_admission = query.admit_relevance();
        let assembly = UiEvidenceSliceAssembly::assemble(
            &query,
            UiEvidenceSliceAssemblyInput::new(
                authority_generation,
                filter_refs_for_query(target.refs, &query),
            )
            .with_materialized_detail(Some(detail))
            .with_detail_available(true)
            .with_cost_metrics(target.cost_metrics),
        );

        Some(UiInspectionReceipt::from_support_and_assembled_slice(
            query,
            relevance_admission,
            support_report,
            authority_generation,
            assembly,
        ))
    }

    fn measurement_view_for_target(
        &self,
        app: &crate::facade::WorthUiApp,
        target: &MeasurementInspectionTarget<'a>,
        support_report: UiInspectionSupportReport,
    ) -> UiInspectionMeasurementEvidenceView {
        if target.graph_node_identity.is_none() && target.graph_node_count > 1 {
            return project_measurement_inspection_denial_view(
                support_report,
                UiInspectionMeasurementDenialPosture::AmbiguousGraphNodeInstances {
                    instance_count: target.graph_node_count,
                },
                Some(UiInspectionMeasurementFailureSource::DeclarationPosture),
            );
        }

        let Some(graph_node_identity) = target.graph_node_identity else {
            return project_measurement_inspection_denial_view(
                support_report,
                UiInspectionMeasurementDenialPosture::MissingEvidence {
                    slot: UiInspectionMeasurementEvidenceSlot::HostCapabilityReport,
                },
                Some(UiInspectionMeasurementFailureSource::DeclarationPosture),
            );
        };

        match self.measurement_outcome_for_target(app, target.artifact, graph_node_identity) {
            MeasurementInspectionOutcome::Basis(basis) => {
                project_measurement_inspection_view(support_report, Some(&basis))
            }
            MeasurementInspectionOutcome::Denial(denial_posture, failure_source) => {
                project_measurement_inspection_denial_view(
                    support_report,
                    denial_posture,
                    failure_source,
                )
            }
            MeasurementInspectionOutcome::Compatibility(compatibility) => {
                project_measurement_inspection_compatibility_view(support_report, compatibility)
            }
        }
    }

    fn measurement_outcome_for_target(
        &self,
        app: &crate::facade::WorthUiApp,
        artifact: &UiDeclarationArtifact,
        graph_node_identity: crate::graph::UiGraphNodeIdentity,
    ) -> MeasurementInspectionOutcome {
        let Some(policy) = measurement_policy_for_artifact(artifact) else {
            return MeasurementInspectionOutcome::Denial(
                UiInspectionMeasurementDenialPosture::MissingEvidence {
                    slot: UiInspectionMeasurementEvidenceSlot::HostCapabilityReport,
                },
                Some(UiInspectionMeasurementFailureSource::DeclarationPosture),
            );
        };
        let Some(touch) = measurement_touch_for_target(app, artifact, graph_node_identity) else {
            return MeasurementInspectionOutcome::Basis(Box::new(admit_measurement_basis(
                artifact.identity().clone(),
                graph_node_identity,
                app.graph_snapshot().world_profile().clone(),
                UiEvidenceAuthorityGeneration::new(app.graph_snapshot().generation().as_u64()),
                policy,
                &[],
            )));
        };
        let bundle = measurement_bundle_for_artifact(self.measurement_evidence, artifact);
        let mut evidence_inputs = measurement_host_inputs(bundle);
        let target = admission_target_for_touch(&touch, bundle);
        let selected = app
            .admission()
            .select_obligations_for_target(&touch, target);
        let measurement_admission = app.admission().admit_measurement_requirement(&selected);

        if let Some(query_outcome) = query_measurement_outcome_for_bundle(
            app,
            &selected,
            measurement_admission.as_ref(),
            bundle,
        ) {
            match query_outcome {
                QueryMeasurementInspectionOutcome::Receipt(receipt) => {
                    evidence_inputs.push(MeasurementEvidenceInput::settled_query_fact(&receipt));
                }
                QueryMeasurementInspectionOutcome::Denial(denial_posture, failure_source) => {
                    return MeasurementInspectionOutcome::Denial(denial_posture, failure_source);
                }
                QueryMeasurementInspectionOutcome::Compatibility(compatibility) => {
                    return MeasurementInspectionOutcome::Compatibility(compatibility);
                }
            }
        }

        if let Some(admission) = measurement_admission.as_ref() {
            if let UiMeasurementAdmissionPosture::Admitted {
                host_capability, ..
            } = admission.posture()
            {
                if bundle.is_none_or(|candidate| candidate.host_capability_report().is_none()) {
                    evidence_inputs.push(MeasurementEvidenceInput::host_capability_report(
                        host_capability,
                    ));
                }
            }
        }

        MeasurementInspectionOutcome::Basis(Box::new(admit_measurement_basis(
            artifact.identity().clone(),
            graph_node_identity,
            app.graph_snapshot().world_profile().clone(),
            UiEvidenceAuthorityGeneration::new(app.graph_snapshot().generation().as_u64()),
            policy,
            &evidence_inputs,
        )))
    }

    fn lookup_target(
        &self,
        target: &UiInspectionTarget,
    ) -> Option<MeasurementInspectionTarget<'a>> {
        match target {
            UiInspectionTarget::DeclaredSurface {
                module_path,
                declaration_index,
            } => {
                let artifact = self.declaration_artifacts.iter().find(|artifact| {
                    let provenance = artifact.provenance().source_provenance();
                    provenance.module_path() == module_path
                        && provenance.declaration_index() == *declaration_index
                })?;
                let (graph_node_identity, graph_node_count) =
                    declaration_instance_resolution(self.graph_snapshot, artifact.identity());
                let lookup = self
                    .authored_evidence_index
                    .lookup_declaration_identity(artifact.identity().inspection_identity())?;
                Some(MeasurementInspectionTarget::new(
                    artifact,
                    graph_node_identity,
                    graph_node_count,
                    lookup.neighborhood().refs(),
                    UiInspectionCostMetrics::new(
                        lookup.cost().index_lookups(),
                        lookup.neighborhood().refs().len(),
                        0,
                        false,
                    ),
                ))
            }
            UiInspectionTarget::DeclarationIdentity { identity } => {
                let lookup = self
                    .authored_evidence_index
                    .lookup_declaration_identity(*identity)?;
                let artifact =
                    &self.declaration_artifacts[lookup.neighborhood().declaration_artifact_index()];
                let (graph_node_identity, graph_node_count) =
                    declaration_instance_resolution(self.graph_snapshot, artifact.identity());
                Some(MeasurementInspectionTarget::new(
                    artifact,
                    graph_node_identity,
                    graph_node_count,
                    lookup.neighborhood().refs(),
                    UiInspectionCostMetrics::new(
                        lookup.cost().index_lookups(),
                        lookup.neighborhood().refs().len(),
                        0,
                        false,
                    ),
                ))
            }
            UiInspectionTarget::AuthoredSourceProvenance { provenance } => {
                let lookup = self
                    .authored_evidence_index
                    .lookup_authored_provenance(provenance)?;
                let artifact =
                    &self.declaration_artifacts[lookup.neighborhood().declaration_artifact_index()];
                let (graph_node_identity, graph_node_count) =
                    declaration_instance_resolution(self.graph_snapshot, artifact.identity());
                Some(MeasurementInspectionTarget::new(
                    artifact,
                    graph_node_identity,
                    graph_node_count,
                    lookup.neighborhood().refs(),
                    UiInspectionCostMetrics::new(
                        lookup.cost().index_lookups(),
                        lookup.neighborhood().refs().len(),
                        0,
                        false,
                    ),
                ))
            }
            UiInspectionTarget::GraphNodeIdentity { graph_node_digest } => {
                let lookup = self.graph_node_evidence_index.lookup_graph_node_identity(
                    crate::graph::UiGraphNodeIdentity::new(*graph_node_digest),
                )?;
                let artifact =
                    &self.declaration_artifacts[lookup.neighborhood().declaration_artifact_index()];
                Some(MeasurementInspectionTarget::new(
                    artifact,
                    Some(crate::graph::UiGraphNodeIdentity::new(*graph_node_digest)),
                    1,
                    lookup.neighborhood().refs(),
                    UiInspectionCostMetrics::new(
                        lookup.cost().index_lookups(),
                        lookup.neighborhood().refs().len(),
                        0,
                        false,
                    ),
                ))
            }
            UiInspectionTarget::ProductRoot
            | UiInspectionTarget::PublishedAspect { .. }
            | UiInspectionTarget::ConsumedAspect { .. }
            | UiInspectionTarget::ObligationGraphNode { .. }
            | UiInspectionTarget::ObligationTouch { .. }
            | UiInspectionTarget::ObligationEvidenceHandle { .. }
            | _ => None,
        }
    }
}
