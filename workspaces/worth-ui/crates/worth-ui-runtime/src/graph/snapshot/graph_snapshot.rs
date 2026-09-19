use crate::declaration::stable_text_digest;
use crate::graph::{
    UiGraphAuthorityIdentity, UiGraphAxisParticipation, UiGraphCoreIndexes,
    UiGraphDeclarationCorrespondence, UiGraphGeneration, UiGraphGenerationRelation,
    UiGraphMountEligibilityMutation, UiGraphMountEligibilitySlot, UiGraphMountEligibilityStore,
    UiGraphMountEligibilityTransition, UiGraphNode, UiGraphNodeIdentity, UiGraphSnapshotComparable,
    UiGraphTopology, UiGraphWorldDifferenceKind, UiGraphWorldProfile,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphSnapshot {
    authority_identity: UiGraphAuthorityIdentity,
    generation: UiGraphGeneration,
    world_profile: UiGraphWorldProfile,
    declaration_authority_digest: u64,
    snapshot_authority_digest: u64,
    nodes: Vec<UiGraphNode>,
    topology: UiGraphTopology,
    mount_eligibilities: UiGraphMountEligibilityStore,
    core_indexes: UiGraphCoreIndexes,
}

impl UiGraphSnapshot {
    pub(crate) fn new(
        generation: UiGraphGeneration,
        world_profile: UiGraphWorldProfile,
        nodes: Vec<UiGraphNode>,
        topology: UiGraphTopology,
        mount_eligibilities: UiGraphMountEligibilityStore,
        core_indexes: UiGraphCoreIndexes,
    ) -> Self {
        let declaration_correspondence = core_indexes.declaration_correspondence();
        let declaration_authority_digest =
            declaration_authority_digest(declaration_correspondence, &world_profile);
        let snapshot_authority_digest = snapshot_authority_digest(
            generation,
            &world_profile,
            &nodes,
            &topology,
            &mount_eligibilities,
            declaration_authority_digest,
        );

        Self {
            authority_identity: UiGraphAuthorityIdentity::mint(),
            generation,
            world_profile,
            declaration_authority_digest,
            snapshot_authority_digest,
            nodes,
            topology,
            mount_eligibilities,
            core_indexes,
        }
    }

    pub fn generation(&self) -> UiGraphGeneration {
        self.generation
    }

    pub fn world_profile(&self) -> &UiGraphWorldProfile {
        &self.world_profile
    }

    pub(crate) fn nodes(&self) -> &[UiGraphNode] {
        &self.nodes
    }

    pub(crate) fn topology(&self) -> &UiGraphTopology {
        &self.topology
    }

    pub(crate) fn mount_eligibilities(&self) -> &UiGraphMountEligibilityStore {
        &self.mount_eligibilities
    }

    pub(crate) fn core_indexes(&self) -> &UiGraphCoreIndexes {
        &self.core_indexes
    }

    pub(crate) fn authority_digest(&self) -> u64 {
        self.snapshot_authority_digest
    }

    pub(crate) fn authority_identity(&self) -> UiGraphAuthorityIdentity {
        self.authority_identity
    }

    pub(crate) fn rebuild_derived_indexes(&mut self) {
        self.core_indexes =
            UiGraphCoreIndexes::rebuild(&self.nodes, &self.topology, &self.mount_eligibilities);
    }

    pub(crate) fn mount_eligibility_slot_for_node(
        &self,
        graph_node_identity: UiGraphNodeIdentity,
    ) -> Option<&UiGraphMountEligibilitySlot> {
        self.core_indexes
            .mount_eligibilities()
            .slot_for_node(&self.mount_eligibilities, graph_node_identity)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn graph_node_ids_for_authored_provenance(
        &self,
        authored_provenance_digest: u64,
    ) -> &[UiGraphNodeIdentity] {
        self.core_indexes
            .declaration_correspondence()
            .graph_node_ids_for_authored_provenance(authored_provenance_digest)
    }

    pub fn mount_eligibility_slot_count(&self) -> usize {
        self.mount_eligibilities.slots().len()
    }

    pub fn mount_eligibility_mutation_for_node(
        &self,
        graph_node_identity: UiGraphNodeIdentity,
        prior_eligibility: UiGraphAxisParticipation,
        next_eligibility: UiGraphAxisParticipation,
    ) -> Option<UiGraphMountEligibilityMutation> {
        self.mount_eligibility_transition_for_node(
            graph_node_identity,
            prior_eligibility,
            next_eligibility,
        )
        .map(UiGraphMountEligibilityMutation::from_transition)
    }

    pub fn mount_eligibility_transition_for_node(
        &self,
        graph_node_identity: UiGraphNodeIdentity,
        prior_eligibility: UiGraphAxisParticipation,
        next_eligibility: UiGraphAxisParticipation,
    ) -> Option<UiGraphMountEligibilityTransition> {
        self.mount_eligibility_slot_for_node(graph_node_identity)
            .and_then(|slot| {
                UiGraphMountEligibilityTransition::from_slot_axis_transition(
                    self.authority_identity(),
                    *slot,
                    prior_eligibility,
                    next_eligibility,
                )
            })
    }

    pub fn compare_to(&self, other: &Self) -> UiGraphSnapshotComparable {
        let generation_relation = self.generation.relation_to(other.generation);
        let kind = if self.declaration_authority_digest == other.declaration_authority_digest {
            if self.world_profile == other.world_profile {
                same_world_difference_kind(
                    self.snapshot_authority_digest,
                    other.snapshot_authority_digest,
                    generation_relation,
                )
            } else if self.world_profile.comparison_family()
                == other.world_profile.comparison_family()
            {
                UiGraphWorldDifferenceKind::SameDeclarationDifferentWorld
            } else {
                UiGraphWorldDifferenceKind::NotComparable
            }
        } else if self.world_profile.comparison_family() == other.world_profile.comparison_family()
        {
            UiGraphWorldDifferenceKind::DifferentDeclarationAuthority
        } else {
            UiGraphWorldDifferenceKind::NotComparable
        };

        UiGraphSnapshotComparable::new(kind, generation_relation, self.generation, other.generation)
    }
}

fn declaration_authority_digest(
    declaration_correspondence: &UiGraphDeclarationCorrespondence,
    world_profile: &UiGraphWorldProfile,
) -> u64 {
    declaration_correspondence.declaration_digests().fold(
        world_profile.comparison_family().rotate_left(7),
        |digest, declaration_digest| digest.rotate_left(5) ^ declaration_digest,
    )
}

fn same_world_difference_kind(
    current_snapshot_digest: u64,
    compared_snapshot_digest: u64,
    generation_relation: UiGraphGenerationRelation,
) -> UiGraphWorldDifferenceKind {
    if current_snapshot_digest == compared_snapshot_digest {
        UiGraphWorldDifferenceKind::SameWorldEquivalent
    } else if matches!(
        generation_relation,
        UiGraphGenerationRelation::DirectSuccessor | UiGraphGenerationRelation::DirectPredecessor
    ) {
        UiGraphWorldDifferenceKind::SameWorldSuccessor
    } else {
        UiGraphWorldDifferenceKind::SameWorldUnrelatedGeneration
    }
}

fn snapshot_authority_digest(
    generation: UiGraphGeneration,
    world_profile: &UiGraphWorldProfile,
    nodes: &[UiGraphNode],
    topology: &UiGraphTopology,
    mount_eligibilities: &UiGraphMountEligibilityStore,
    declaration_authority_digest: u64,
) -> u64 {
    nodes.iter().fold(
        stable_text_digest("graph-snapshot")
            ^ generation.as_u64().rotate_left(5)
            ^ declaration_authority_digest.rotate_left(17)
            ^ topology.identity_digest().rotate_left(23)
            ^ mount_eligibilities.identity_digest().rotate_left(31)
            ^ world_profile.identity_digest().rotate_left(29),
        |digest, node| digest.rotate_left(7) ^ node.authority_digest(),
    )
}

#[cfg(test)]
mod tests {
    use crate::declaration::{
        UiDeclarationArtifact, UiDeclarationGraphHandoff, UiDeclaredAspectPayload,
        UiDeclaredPostureContract, UiDeclaredPostureLane, UiDeclaredPosturePayload,
        UiStructuralDeclarationPayload,
    };
    use crate::facade::WorthUiRustAuthoredDeclarationFixture;
    use crate::facade::{WorthUi, WorthUiApp};
    use crate::graph::{
        UiGraphGeneration, UiGraphInstantiationPlan, UiGraphWorldDifferenceKind,
        UiGraphWorldProfile,
    };
    use worth_ui_dsl::{
        UiDslPostureToken, UiDslSemanticArtifactSpec, UiDslSemanticFamily, UiDslSemanticKey,
        UiDslSourceProvenance, UiDslStructuralToken,
    };

    use super::UiGraphSnapshot;

    #[test]
    fn same_world_successor_requires_explicit_generation_lineage() {
        let app = snapshot_fixture_app(true);
        let initial = committed_snapshot_fixture(&app, true);
        let successor = UiGraphSnapshot::new(
            UiGraphGeneration::successor_of(initial.generation()),
            UiGraphWorldProfile::authoritative(),
            initial.nodes().to_vec(),
            initial.topology().clone(),
            initial.mount_eligibilities().clone(),
            initial.core_indexes().clone(),
        );

        assert_eq!(successor.core_indexes(), initial.core_indexes());

        assert_eq!(
            successor.compare_to(&initial).kind(),
            UiGraphWorldDifferenceKind::SameWorldSuccessor
        );
    }

    #[test]
    fn child_bounded_posture_changes_same_world_snapshot_equivalence() {
        let app = snapshot_fixture_app(true);
        let bounded = committed_snapshot_fixture(&app, true);
        let unbounded = committed_snapshot_fixture(&app, false);

        assert_eq!(
            bounded.compare_to(&unbounded).kind(),
            UiGraphWorldDifferenceKind::SameWorldUnrelatedGeneration
        );
    }

    fn snapshot_fixture_app(child_bounded: bool) -> WorthUiApp {
        let mut child = UiDslSemanticArtifactSpec::new(
            UiDslSemanticKey::new("ui.graph.snapshot.child"),
            UiDslSemanticFamily::Control,
            UiDslSourceProvenance::file_authored("app/graph_snapshot_tests.wui", 0),
        )
        .with_structural_token(UiDslStructuralToken::new("control:child"))
        .with_structural_token(UiDslStructuralToken::new("slot:footer"));
        if child_bounded {
            child =
                child.with_posture_token(UiDslPostureToken::new("measurement:constraint:bounded"));
        }

        WorthUi::app()
            .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
            .with_rust_authored_declaration_fixture(
                WorthUiRustAuthoredDeclarationFixture::named("worth-ui.runtime.graph.tests")
                    .with_semantic_artifact_spec(child),
            )
            .freeze()
            .map(
                crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host,
            )
            .expect("application preparation should succeed")
    }

    fn committed_snapshot_fixture(app: &WorthUiApp, child_bounded: bool) -> UiGraphSnapshot {
        let root_page = root_page_artifact(app)
            .graph_handoff()
            .expect("root page should lower to graph handoff");
        let child = child_handoff(app, child_bounded);
        UiGraphInstantiationPlan::admit_handoffs(&[root_page, child], &[])
            .expect("graph handoffs should admit into instantiation plan")
            .commit_initial_generation(UiGraphWorldProfile::authoritative())
            .expect("instantiation plan should commit without local denials")
            .into_committed_snapshot()
    }

    fn child_handoff(app: &WorthUiApp, child_bounded: bool) -> UiDeclarationGraphHandoff {
        let artifact = artifact_from_file_provenance(app, "app/graph_snapshot_tests.wui", 0);
        let graph_handoff = artifact
            .graph_handoff()
            .expect("child declaration should lower to graph handoff");
        if child_bounded {
            return graph_handoff;
        }

        let declared_posture = artifact
            .declared_posture()
            .expect("child declaration posture should be admitted");
        let measurement_policy = declared_posture.measurement_policy();
        let measurement_admitted = measurement_policy.admitted().and_then(|policy| {
            crate::declaration::UiDeclaredMeasurementPolicyPosture::new(
                policy.mode(),
                None,
                policy.basis_source(),
                policy.ownership_posture(),
                policy.evidence_requirements().to_vec(),
            )
        });
        let declared_posture = UiDeclaredPostureContract::new(
            declared_posture.query_binding().clone(),
            declared_posture.service_usage().clone(),
            declared_posture.touch_meaning().clone(),
            UiDeclaredPostureLane::new(measurement_policy.applicability(), measurement_admitted),
            declared_posture.host_capability().clone(),
        );

        UiDeclarationGraphHandoff::new(
            graph_handoff.identity().clone(),
            graph_handoff.authored_provenance_digest(),
            UiStructuralDeclarationPayload::new(
                graph_handoff.family().clone(),
                graph_handoff.structural_digest(),
                graph_handoff.structural_semantics().clone(),
            ),
            UiDeclaredAspectPayload::new(graph_handoff.aspect_contract().clone()),
            UiDeclaredPosturePayload::new(declared_posture),
            graph_handoff.component_reference().cloned(),
            graph_handoff.appearance_role_attachment().cloned(),
        )
    }

    fn artifact_from_file_provenance<'a>(
        app: &'a WorthUiApp,
        module_path: &str,
        declaration_index: usize,
    ) -> &'a UiDeclarationArtifact {
        app.declaration_artifacts()
            .iter()
            .find(|artifact| {
                let provenance = artifact.provenance().source_provenance();
                provenance.module_path() == module_path
                    && provenance.declaration_index() == declaration_index
            })
            .unwrap_or_else(|| {
                panic!(
                    "expected declaration artifact for {module_path}#{declaration_index} on freeze path"
                )
            })
    }

    fn root_page_artifact(app: &WorthUiApp) -> &UiDeclarationArtifact {
        app.declaration_artifacts()
            .iter()
            .find(|artifact| {
                artifact
                    .graph_handoff()
                    .map(|handoff| {
                        handoff.role() == crate::declaration::UiDeclarationStructuralRole::Page
                    })
                    .unwrap_or(false)
            })
            .expect("bootstrap root page artifact should exist")
    }
}
