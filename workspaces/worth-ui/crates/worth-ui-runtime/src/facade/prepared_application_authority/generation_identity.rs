use super::application_artifact::WorthUiPreparedApplicationArtifactIdentity;
use super::query_binding_plan_identity::WorthUiPreparedQueryBindingPlanIdentity;
use super::WorthUiPreparedDeclarationSourceIdentity;
use crate::capability::CapabilitySnapshotDigest;
use std::rc::Rc;

/// Comparison-safe identity of exactly one prepared application generation.
/// No public constructor accepts component digests or promotes candidate truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPreparedApplicationGenerationIdentity {
    inner: Rc<WorthUiPreparedApplicationGenerationIdentityInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiPreparedApplicationGenerationIdentityInner {
    capability_snapshot: CapabilitySnapshotDigest,
    canonical_artifact: WorthUiPreparedApplicationArtifactIdentity,
    lineage: WorthUiPreparedGenerationLineage,
    declaration_source: WorthUiPreparedDeclarationSourceIdentity,
    semantic_package: worth_ui_dsl::WorthUiSemanticPackageIdentity,
    graph_authority_digest: u64,
    query_binding: WorthUiPreparedQueryBindingPlanIdentity,
    intent_application_fact_digest: u64,
    intent_execution_binding_digest: u64,
    service_policy_digest: u64,
    visual_inspection_policy: worth_ui_inspection::UiVisualInspectionPolicy,
    change_profile: crate::runtime::rebind::UiChangeProfile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiPreparedGenerationLineage {
    InitialPreparation,
    AuthoredSourceSuccessor(crate::runtime::WorthUiAuthoredSourceBasis),
}

impl WorthUiPreparedGenerationLineage {
    pub(crate) const fn initial() -> Self {
        Self::InitialPreparation
    }

    pub(crate) fn authored_source_successor(
        basis: crate::runtime::WorthUiAuthoredSourceBasis,
    ) -> Self {
        Self::AuthoredSourceSuccessor(basis)
    }
}

pub(super) struct WorthUiPreparedGenerationIdentityInput<'plan> {
    pub(super) capability_snapshot: CapabilitySnapshotDigest,
    pub(super) canonical_artifact: WorthUiPreparedApplicationArtifactIdentity,
    pub(super) lineage: WorthUiPreparedGenerationLineage,
    pub(super) declaration_source: WorthUiPreparedDeclarationSourceIdentity,
    pub(super) semantic_package: worth_ui_dsl::WorthUiSemanticPackageIdentity,
    pub(super) graph_authority_digest: u64,
    pub(super) query_binding_plan: &'plan worth_ui_query_binding::WorthUiQueryBindingPlan,
    pub(super) intent_application_fact_digest: u64,
    pub(super) intent_execution_binding_digest: u64,
    pub(super) service_policy_digest: u64,
    pub(super) visual_inspection_policy: worth_ui_inspection::UiVisualInspectionPolicy,
    pub(super) change_profile: crate::runtime::rebind::UiChangeProfile,
}

impl WorthUiPreparedApplicationGenerationIdentity {
    pub(super) fn derive(input: WorthUiPreparedGenerationIdentityInput<'_>) -> Self {
        Self {
            inner: Rc::new(WorthUiPreparedApplicationGenerationIdentityInner {
                capability_snapshot: input.capability_snapshot,
                canonical_artifact: input.canonical_artifact,
                lineage: input.lineage,
                declaration_source: input.declaration_source,
                semantic_package: input.semantic_package,
                graph_authority_digest: input.graph_authority_digest,
                query_binding: WorthUiPreparedQueryBindingPlanIdentity::derive(
                    input.query_binding_plan,
                ),
                intent_application_fact_digest: input.intent_application_fact_digest,
                intent_execution_binding_digest: input.intent_execution_binding_digest,
                service_policy_digest: input.service_policy_digest,
                visual_inspection_policy: input.visual_inspection_policy,
                change_profile: input.change_profile,
            }),
        }
    }

    pub fn semantic_package_identity(&self) -> &worth_ui_dsl::WorthUiSemanticPackageIdentity {
        &self.inner.semantic_package
    }

    pub(crate) fn matches_graph(&self, graph: crate::graph::UiGraphAuthority<'_>) -> bool {
        self.inner.graph_authority_digest == graph.snapshot().authority_digest()
    }
}
