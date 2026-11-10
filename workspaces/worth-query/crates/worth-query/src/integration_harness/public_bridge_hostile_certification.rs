use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::{
    authoring::{
        AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath, RawAuthoredQuery,
        RawAuthoredResultShape, RootEntityKey,
    },
    authorized_projection::{
        derive_authorized_projection, AuthorizedProjectionArtifact, PolicyAspectMask,
        PolicyInfluenceSet,
    },
    canonicalization::CanonicalResultShapeArtifact,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicBridgeHostileCertificationComposeInput {
    pub pending_artifact: String,
    pub branch_basis_a: String,
    pub branch_basis_b: String,
    pub preview_discard: String,
    pub receipt_one: WorthQueryEvidenceIdentity,
    pub title_one: String,
    pub receipt_two: WorthQueryEvidenceIdentity,
    pub title_two: String,
    pub preview_promote: String,
    pub title_three: String,
}

pub fn public_bridge_hostile_certification_evidence_label(
    identity: &WorthQueryEvidenceIdentity,
) -> String {
    identity.reporting_projection().to_string()
}

pub fn public_bridge_hostile_published_artifact_component_digest(
    snapshot_identity: &WorthQueryEvidenceIdentity,
    binding_for_reporting: &str,
    title: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(public_bridge_hostile_certification_evidence_label(
        &worth_query_evidence_identity(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), snapshot_identity)
        .field_value(WorthQueryEvidenceTag::new("binding"), binding_for_reporting)
        .field_shape(WorthQueryEvidenceTag::new("title"), title)
        .seal()?,
    ))
}

pub fn compose_public_bridge_hostile_certification_digest(
    input: PublicBridgeHostileCertificationComposeInput,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact)
        .field_value(
            WorthQueryEvidenceTag::new("pending_artifact"),
            input.pending_artifact,
        )
        .field_value(
            WorthQueryEvidenceTag::new("branch_basis_a"),
            input.branch_basis_a,
        )
        .field_value(
            WorthQueryEvidenceTag::new("branch_basis_b"),
            input.branch_basis_b,
        )
        .field_value(
            WorthQueryEvidenceTag::new("preview_discard"),
            input.preview_discard,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt_one"),
            &input.receipt_one,
        )
        .field_shape(WorthQueryEvidenceTag::new("title_one"), input.title_one)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt_two"),
            &input.receipt_two,
        )
        .field_shape(WorthQueryEvidenceTag::new("title_two"), input.title_two)
        .field_value(
            WorthQueryEvidenceTag::new("preview_promote"),
            input.preview_promote,
        )
        .field_shape(WorthQueryEvidenceTag::new("title_three"), input.title_three)
        .seal()
}

pub fn public_bridge_hostile_title_projection_artifacts(
) -> (CanonicalResultShapeArtifact, AuthorizedProjectionArtifact) {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("task").unwrap())
        .project(AspectFieldSelector::new("title", "value").unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("title", "value", "title.value").unwrap())
        .build()
        .unwrap();
    let canonical = GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap();
    let authorized_projection = derive_authorized_projection(
        canonical.query(),
        canonical.result_shape(),
        "policy:public-bridge-hostile-certification",
        "schema:public-bridge-hostile-certification",
        &PolicyAspectMask::allow_all(),
        &PolicyInfluenceSet::none(),
        8,
        8,
    )
    .unwrap();
    (canonical.result_shape().clone(), authorized_projection)
}

pub fn public_bridge_projection_artifacts_for_canonical_bundle(
    canonical: &crate::facade::foundation::CanonicalQueryBundle,
) -> (CanonicalResultShapeArtifact, AuthorizedProjectionArtifact) {
    let authorized_projection = derive_authorized_projection(
        canonical.query(),
        canonical.result_shape(),
        "policy:public-bridge-hostile-certification",
        "schema:public-bridge-hostile-certification",
        &PolicyAspectMask::allow_all(),
        &PolicyInfluenceSet::none(),
        8,
        8,
    )
    .unwrap();
    (canonical.result_shape().clone(), authorized_projection)
}

pub fn public_bridge_projection_artifacts_for_declarative_request(
    request: &crate::facade::foundation::DeclarativeLiveQueryRequest,
) -> (CanonicalResultShapeArtifact, AuthorizedProjectionArtifact) {
    let canonical = crate::declarative_live::canonicalize_declarative_request(request).unwrap();
    public_bridge_projection_artifacts_for_canonical_bundle(&canonical)
}

pub fn public_bridge_projection_artifacts_for_read_graph(
    read_graph: &crate::runtime::WorthQueryReadGraph,
) -> (CanonicalResultShapeArtifact, AuthorizedProjectionArtifact) {
    let canonical =
        crate::declarative_live::canonicalize_declarative_request(read_graph.declarative_request())
            .unwrap();
    let derived = public_bridge_projection_artifacts_for_canonical_bundle(&canonical).1;
    let authorized_projection = AuthorizedProjectionArtifact::new(
        read_graph.query_digest(),
        derived.result_shape_digest(),
        derived.policy_digest(),
        derived.tenant_schema_basis_digest(),
        derived.visible_field_paths().to_vec(),
        derived.masked_projection().clone(),
        derived.narrowed_result_shape_digest().to_string(),
        derived.influence_set().clone(),
        derived.counters().clone(),
    );
    (canonical.result_shape().clone(), authorized_projection)
}
