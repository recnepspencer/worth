use worth_ui_host_contract::{UiMeasurementEvidenceFamily, UiMeasurementRequestIdentity};
use worth_ui_inspection::UiEvidenceAuthorityGeneration;

mod partition;

use partition::{disjoint_partition, required_host_families};

use super::{
    mounted_allocation_denial::map_initial_activation_denial, WorthUiActiveApplicationSession,
    WorthUiMountedAllocationEstablishmentDenial, WorthUiMountedAllocationRuntimeStage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedAllocationMeasurementRequest {
    pub(super) evidence_family: UiMeasurementEvidenceFamily,
    pub(super) need: crate::host::UiHostMeasurementNeed,
    pub(super) normalization_context: crate::host::UiHostMeasurementNormalizationContext,
}

#[derive(Debug)]
pub struct WorthUiMountedAllocationEstablishmentReceipt {
    committed: crate::runtime::UiCommittedAllocationReplan,
}

struct MountedAllocationCandidate {
    declaration: crate::declaration::UiDeclarationIdentity,
    node: crate::graph::UiGraphNodeIdentity,
    selected: crate::obligations::selection::UiSelectedObligationSet,
    transition: crate::graph::UiGraphMountEligibilityTransition,
    policy: crate::declaration::UiDeclaredMeasurementPolicyPosture,
}

impl UiMountedAllocationMeasurementRequest {
    pub fn new(
        evidence_family: UiMeasurementEvidenceFamily,
        need: crate::host::UiHostMeasurementNeed,
        normalization_context: crate::host::UiHostMeasurementNormalizationContext,
    ) -> Self {
        Self {
            evidence_family,
            need,
            normalization_context,
        }
    }
}

impl WorthUiMountedAllocationEstablishmentReceipt {
    pub fn committed(&self) -> &crate::runtime::UiCommittedAllocationReplan {
        &self.committed
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub fn committed_basis_sources(
        &self,
    ) -> Box<[Option<crate::declaration::UiDeclaredMeasurementBasisSource>]> {
        self.committed
            .receipts()
            .iter()
            .map(|receipt| {
                receipt
                    .committed_allocation()
                    .measurement_basis()
                    .declared_measurement_policy()
                    .basis_source()
            })
            .collect()
    }
}

/// SUPPORT AUTHORITY for certification worlds that establish allocation
/// independently of the ordinary mounted-frame entry.
pub trait WorthUiMountedAllocationCertificationExt {
    fn establish_mounted_allocation_catalog(
        &mut self,
        request_identity_start: u64,
        requests: impl Into<Box<[UiMountedAllocationMeasurementRequest]>>,
    ) -> Result<
        WorthUiMountedAllocationEstablishmentReceipt,
        WorthUiMountedAllocationEstablishmentDenial,
    >;
}

impl WorthUiActiveApplicationSession {
    pub(crate) fn establish_native_viewport_allocation(
        &mut self,
    ) -> Result<(), WorthUiMountedAllocationEstablishmentDenial> {
        let request = {
            let capability = self.host_measurement_capability();
            let assumptions =
                crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
                    capability.capability_report(),
                    1,
                    2,
                    3,
                    4,
                );
            UiMountedAllocationMeasurementRequest::new(
                UiMeasurementEvidenceFamily::ViewportExtent,
                crate::host::UiHostMeasurementNeed::ViewportExtent(
                    worth_ui_host_contract::UiViewportExtentRequest,
                ),
                crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(
                    assumptions,
                ),
            )
        };
        self.establish_mounted_allocation_catalog(1, [request])?;
        Ok(())
    }

    pub(crate) fn establish_mounted_allocation_catalog(
        &mut self,
        request_identity_start: u64,
        requests: impl Into<Box<[UiMountedAllocationMeasurementRequest]>>,
    ) -> Result<
        WorthUiMountedAllocationEstablishmentReceipt,
        WorthUiMountedAllocationEstablishmentDenial,
    > {
        if self.mounted.has_active_presentation_attempt() {
            return Err(WorthUiMountedAllocationEstablishmentDenial::PresentationInFlight);
        }
        let candidates = self.mounted_allocation_candidates()?;
        let transitions = candidates
            .iter()
            .map(|candidate| candidate.transition)
            .collect();
        let graph_commit = self
            .application
            .graph()
            .commit_mount_eligibility_admissions(transitions)
            .map_err(WorthUiMountedAllocationEstablishmentDenial::GraphMountEligibility)?;
        let graph_successor = self
            .application
            .prepare_graph_successor(graph_commit)
            .map_err(|_| WorthUiMountedAllocationEstablishmentDenial::StaleGraphSuccessor)?;
        let appearance_succession = self
            .prepare_appearance_generation_succession(&graph_successor.generation_succession())
            .map_err(|denial| match denial {
                super::UiAppearanceGenerationSuccessionDenial::Theme(denial) => {
                    WorthUiMountedAllocationEstablishmentDenial::AppearanceThemeSuccession(denial)
                }
                super::UiAppearanceGenerationSuccessionDenial::Inspection(denial) => {
                    WorthUiMountedAllocationEstablishmentDenial::AppearanceInspectionSuccession(
                        denial,
                    )
                }
            })?;
        let entries = self.collect_mounted_measurement_entries(
            graph_successor.graph_snapshot(),
            &candidates,
            request_identity_start,
            &requests.into(),
        )?;
        let admitted = graph_successor
            .graph_snapshot()
            .admit_allocation_catalog_basis_set(entries)
            .map_err(WorthUiMountedAllocationEstablishmentDenial::CatalogAdmission)?;
        let boundary = self
            .application
            .prepare_empty_activation_boundary()
            .map_err(|_| {
                WorthUiMountedAllocationEstablishmentDenial::Runtime(
                    WorthUiMountedAllocationRuntimeStage::CatalogPreparation,
                )
            })?;
        let committed = self
            .application
            .activate_initial_mounted_allocation_catalog(graph_successor, admitted, boundary)
            .map_err(map_initial_activation_denial)?;
        self.commit_appearance_generation_succession(appearance_succession);
        Ok(WorthUiMountedAllocationEstablishmentReceipt { committed })
    }

    fn mounted_allocation_candidates(
        &self,
    ) -> Result<Vec<MountedAllocationCandidate>, WorthUiMountedAllocationEstablishmentDenial> {
        let mut candidates = Vec::new();
        for node in self.application.graph().node_identities() {
            let record = self
                .application
                .graph()
                .lookup()
                .graph_node(node)
                .expect("active graph node remains addressable");
            let declaration = record.value().declaration_identity().clone();
            let Some(policy) = self.application.measurement_policy_for(&declaration) else {
                continue;
            };
            let handle = self
                .mounted_graph_node(node)
                .map_err(WorthUiMountedAllocationEstablishmentDenial::MountedIdentity)?;
            let mounted = self
                .mounted_instances_for(handle)
                .map_err(WorthUiMountedAllocationEstablishmentDenial::MountedIdentity)?;
            if mounted.is_empty() {
                return Err(
                    WorthUiMountedAllocationEstablishmentDenial::MissingMountedInstance(node),
                );
            }
            let touch = self.try_allocation_touch_for_node(node).map_err(|_| {
                WorthUiMountedAllocationEstablishmentDenial::Runtime(
                    WorthUiMountedAllocationRuntimeStage::CatalogPreparation,
                )
            })?;
            let selected = self.admission().select_obligations(&touch);
            let prior = record
                .value()
                .participation_posture()
                .axis(crate::graph::UiGraphParticipationAxis::Mounted);
            let transition = self
                .application
                .graph()
                .mount_eligibility_transition_for_node(
                    node,
                    prior,
                    crate::graph::UiGraphAxisParticipation::runtime_mutation(
                        crate::graph::UiGraphParticipationStatus::Admitted,
                    ),
                )
                .ok_or(WorthUiMountedAllocationEstablishmentDenial::Runtime(
                    WorthUiMountedAllocationRuntimeStage::CatalogPreparation,
                ))?;
            candidates.push(MountedAllocationCandidate {
                declaration,
                node,
                selected,
                transition,
                policy,
            });
        }
        if candidates.is_empty() {
            Err(WorthUiMountedAllocationEstablishmentDenial::NoAllocationPlanningNodes)
        } else {
            Ok(candidates)
        }
    }

    fn collect_mounted_measurement_entries(
        &self,
        graph: &crate::graph::UiGraphSnapshot,
        candidates: &[MountedAllocationCandidate],
        request_identity_start: u64,
        requests: &[UiMountedAllocationMeasurementRequest],
    ) -> Result<
        Vec<(
            crate::evidence::UiMeasurementBasis,
            crate::obligations::selection::UiSelectedObligationSet,
        )>,
        WorthUiMountedAllocationEstablishmentDenial,
    > {
        let capability = self.host_session.measurement_capability();
        let collector = self.application.host_measurement_collector();
        let generation = UiEvidenceAuthorityGeneration::new(graph.generation().as_u64());
        let mut entries = Vec::with_capacity(candidates.len());
        let mut request_ordinal = 0_u64;
        for candidate in candidates {
            let mut inputs = vec![
                crate::evidence::MeasurementEvidenceInput::host_capability_report(
                    capability.capability_report(),
                ),
            ];
            for family in required_host_families(&candidate.policy) {
                let request = requests
                    .iter()
                    .find(|request| request.evidence_family == family)
                    .ok_or(
                        WorthUiMountedAllocationEstablishmentDenial::MissingMeasurementRequest(
                            family,
                        ),
                    )?;
                let identity = request_identity_start
                    .checked_add(request_ordinal)
                    .ok_or(
                        WorthUiMountedAllocationEstablishmentDenial::MeasurementRequestIdentityExhausted,
                    )?;
                request_ordinal = request_ordinal.checked_add(1).ok_or(
                    WorthUiMountedAllocationEstablishmentDenial::MeasurementRequestIdentityExhausted,
                )?;
                let result = collector
                    .collect(
                        capability.adapter(),
                        crate::host::UiHostMeasurementCollectionInput {
                            identity: UiMeasurementRequestIdentity::new(identity),
                            evidence_family: request.evidence_family,
                            need: request.need.clone(),
                            capability_report: capability.capability_report(),
                            evidence_generation: generation,
                            normalization_context: request.normalization_context,
                        },
                    )
                    .map_err(WorthUiMountedAllocationEstablishmentDenial::HostMeasurement)?;
                inputs.push(
                    crate::evidence::MeasurementEvidenceInput::host_measurement_result(&result),
                );
            }
            let basis = crate::evidence::admit_measurement_basis(
                candidate.declaration.clone(),
                candidate.node,
                graph.world_profile().clone(),
                generation,
                &candidate.policy,
                &inputs,
            );
            if let Some(denial) = basis.denial_posture().cloned() {
                return Err(
                    WorthUiMountedAllocationEstablishmentDenial::MeasurementBasis {
                        node: candidate.node,
                        denial,
                    },
                );
            }
            entries.push((basis, candidate.selected.clone()));
        }
        disjoint_partition(graph, entries)
    }
}

impl WorthUiMountedAllocationCertificationExt for WorthUiActiveApplicationSession {
    fn establish_mounted_allocation_catalog(
        &mut self,
        request_identity_start: u64,
        requests: impl Into<Box<[UiMountedAllocationMeasurementRequest]>>,
    ) -> Result<
        WorthUiMountedAllocationEstablishmentReceipt,
        WorthUiMountedAllocationEstablishmentDenial,
    > {
        WorthUiActiveApplicationSession::establish_mounted_allocation_catalog(
            self,
            request_identity_start,
            requests,
        )
    }
}
