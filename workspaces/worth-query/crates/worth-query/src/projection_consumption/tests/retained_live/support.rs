use std::sync::OnceLock;

use std::collections::BTreeMap;

use crate::authorized_projection::{
    AuthorizedProjectionArtifact, AuthorizedProjectionCounters, MaskedProjectionArtifact,
    PolicyFieldInfluenceSet,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::{WorthQueryEntity, WorthQuerySnapshotIdentity};
use crate::projection_consumption::{ProjectMaterializedFacts, ProjectionFactKind};
use crate::runtime::{
    WorthQueryDerivedMaterializationBundle, WorthQueryDerivedMaterializationReceipt,
    WorthQueryDerivedMaterializationResult, WorthQueryDerivedMaterializationTarget,
    WorthQueryLiveArtifactBundle, WorthQueryLiveArtifactTarget, WorthQueryLiveReadReceipt,
    WorthQueryLiveReadResult, WorthQueryRetainedFieldPath, WorthQueryRetainedMaterializedRow,
};
use worth_foundational::facade::{
    AspectKey, AspectValue, CanonicalFieldPath, FieldKey, StructAspectValue,
};
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;

pub(super) struct SharedTestResultShape {
    pub identity: WorthQueryEvidenceIdentity,
    pub digest: String,
}

pub(super) fn shared_test_result_shape() -> &'static SharedTestResultShape {
    static SHARED: OnceLock<SharedTestResultShape> = OnceLock::new();
    SHARED.get_or_init(|| {
        let identity = test_result_shape_identity("result-shape:test");
        let digest = identity.as_str().to_string();
        SharedTestResultShape { identity, digest }
    })
}
pub(super) fn test_result_shape_identity(label: &str) -> WorthQueryEvidenceIdentity {
    crate::identity::canonical_result_shape_evidence_identity(
        test_result_shape_artifact(label).result_shape_identity(),
    )
    .expect("fixture digest must fit")
}

pub(super) fn test_result_shape_artifact(
    label: &str,
) -> crate::canonicalization::CanonicalResultShapeArtifact {
    use crate::authoring::ResultShapeFamily;
    use crate::canonicalization::CanonicalResultShapeArtifact;
    use crate::identity::CanonicalResultShapeDigest;

    CanonicalResultShapeArtifact {
        digest: CanonicalResultShapeDigest::from_parts(&[label.to_string()]),
        family: ResultShapeFamily::Detail,
        fields: Vec::new(),
    }
}

pub(super) fn test_result_shape_canonical_digest(label: &str) -> String {
    test_result_shape_artifact(label)
        .digest()
        .as_str()
        .to_string()
}

fn test_live_installation_identity(label: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "test_runtime_live_subscription_installation_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("installation"), label)
        .seal()
        .expect("fixture digest must fit")
}
pub(super) fn authorized_projection(
    query_digest: &str,
    result_shape_digest: &str,
    visible_fields: &[&str],
) -> AuthorizedProjectionArtifact {
    AuthorizedProjectionArtifact::new(
        query_digest,
        result_shape_digest,
        "policy:test",
        "tenant-schema:test",
        crate::projection_consumption::test_authorized_field_paths(visible_fields),
        MaskedProjectionArtifact::new(Vec::new(), Vec::new()),
        "narrowed-result-shape:test".to_string(),
        PolicyFieldInfluenceSet::new(&["influence:test".to_string()], 1),
        AuthorizedProjectionCounters::default(),
    )
}

pub(super) fn retained_binding() -> crate::runtime::WorthQueryDerivedArtifactBinding {
    let retained_snapshot = retained_live_snapshot_identity("snapshot-retained");
    let first = WorthQueryDerivedMaterializationTarget::test_only("derived.first");
    let second = WorthQueryDerivedMaterializationTarget::test_only("derived.second");
    let bundle = WorthQueryDerivedMaterializationBundle::test_only(
        retained_snapshot.clone(),
        BTreeMap::from([
            (
                first.clone(),
                WorthQueryDerivedMaterializationResult::test_only_retained_rows(
                    vec![
                        retained_materialized_row([
                            ("profile.display_name", text_value("First")),
                            ("metrics.priority", AspectValue::Int64(1)),
                        ]),
                        retained_materialized_row([
                            ("profile.display_name", text_value("Second")),
                            ("metrics.priority", AspectValue::Int64(2)),
                        ]),
                    ],
                    WorthQueryDerivedMaterializationReceipt::test_only(
                        first.view_name(),
                        retained_snapshot.clone(),
                        "derived-first-digest",
                    ),
                ),
            ),
            (
                second.clone(),
                WorthQueryDerivedMaterializationResult::test_only_retained_rows(
                    vec![retained_materialized_row([(
                        "profile.display_name",
                        text_value("Third"),
                    )])],
                    WorthQueryDerivedMaterializationReceipt::test_only(
                        second.view_name(),
                        retained_snapshot.clone(),
                        "derived-second-digest",
                    ),
                ),
            ),
        ]),
    )
    .expect("fixture digest must fit");

    bundle
        .bind_retained_artifact("retained.binding", [first.clone(), second.clone()])
        .expect("retained binding should succeed")
}

pub(super) fn live_binding() -> crate::runtime::WorthQueryLiveArtifactBinding {
    let live_snapshot = retained_live_snapshot_identity("snapshot-live");
    let first = WorthQueryLiveArtifactTarget::test_only("live.first");
    let second = WorthQueryLiveArtifactTarget::test_only("live.second");
    let bundle = WorthQueryLiveArtifactBundle::test_only(
        live_snapshot.clone(),
        BTreeMap::from([
            (
                first.clone(),
                WorthQueryLiveReadResult::test_only(
                    vec![
                        WorthQueryEntity::from_native_field_values(
                            crate::memory_workspace::admit_authored_entity_label("entity-1")
                                .expect("fixture digest must fit"),
                            projection_values([("profile.display_name", text_value("First"))]),
                        ),
                        WorthQueryEntity::from_native_field_values(
                            crate::memory_workspace::admit_authored_entity_label("entity-2")
                                .expect("fixture digest must fit"),
                            projection_values([("profile.display_name", text_value("Second"))]),
                        ),
                    ],
                    WorthQueryLiveReadReceipt::test_only(
                        first.view_name(),
                        "installation:first",
                        test_live_installation_identity("installation:first"),
                        "query:test",
                        test_result_shape_artifact("shape:first").digest().clone(),
                        "subscription:first",
                        "result:first",
                        live_snapshot.clone(),
                        2,
                    )
                    .expect("fixture digest must fit"),
                ),
            ),
            (
                second.clone(),
                WorthQueryLiveReadResult::test_only(
                    vec![WorthQueryEntity::from_native_field_values(
                        crate::memory_workspace::admit_authored_entity_label("entity-3")
                            .expect("fixture digest must fit"),
                        projection_values([("profile.display_name", text_value("Third"))]),
                    )],
                    WorthQueryLiveReadReceipt::test_only(
                        second.view_name(),
                        "installation:second",
                        test_live_installation_identity("installation:second"),
                        "query:test",
                        test_result_shape_artifact("shape:second").digest().clone(),
                        "subscription:second",
                        "result:second",
                        live_snapshot.clone(),
                        1,
                    )
                    .expect("fixture digest must fit"),
                ),
            ),
        ]),
    )
    .expect("fixture digest must fit");

    bundle
        .bind_live_artifact("live.binding", [first.clone(), second.clone()])
        .expect("live binding should succeed")
}

pub(super) fn retained_struct_binding() -> crate::runtime::WorthQueryDerivedArtifactBinding {
    let snapshot = retained_live_snapshot_identity("snapshot-retained-struct");
    let target = WorthQueryDerivedMaterializationTarget::test_only("derived.struct");
    let row = WorthQueryRetainedMaterializedRow::from_native_values(
        BTreeMap::new(),
        BTreeMap::from([(retained_field_path("profile"), profile_struct_value())]),
    )
    .unwrap();
    let bundle = WorthQueryDerivedMaterializationBundle::test_only(
        snapshot.clone(),
        BTreeMap::from([(
            target.clone(),
            WorthQueryDerivedMaterializationResult::test_only_retained_rows(
                vec![row],
                WorthQueryDerivedMaterializationReceipt::test_only(
                    target.view_name(),
                    snapshot,
                    "derived-struct-digest",
                ),
            ),
        )]),
    )
    .expect("fixture digest must fit");
    bundle
        .bind_retained_artifact("retained.struct.binding", [target])
        .unwrap()
}

pub(super) fn live_struct_binding() -> crate::runtime::WorthQueryLiveArtifactBinding {
    let snapshot = retained_live_snapshot_identity("snapshot-live-struct");
    let target = WorthQueryLiveArtifactTarget::test_only("live.struct");
    let row = WorthQueryEntity::from_aspect_projection(
        crate::memory_workspace::admit_authored_entity_label("entity-struct")
            .expect("fixture digest must fit"),
        BTreeMap::new(),
        BTreeMap::from([(AspectKey::new("profile").unwrap(), profile_struct_value())]),
        BTreeMap::new(),
    );
    let bundle = WorthQueryLiveArtifactBundle::test_only(
        snapshot.clone(),
        BTreeMap::from([(
            target.clone(),
            WorthQueryLiveReadResult::test_only(
                vec![row],
                WorthQueryLiveReadReceipt::test_only(
                    target.view_name(),
                    "installation:struct",
                    test_live_installation_identity("installation:struct"),
                    "query:test",
                    test_result_shape_artifact("shape:struct").digest().clone(),
                    "subscription:struct",
                    "result:struct",
                    snapshot,
                    1,
                )
                .expect("fixture digest must fit"),
            ),
        )]),
    )
    .expect("fixture digest must fit");
    bundle
        .bind_live_artifact("live.struct.binding", [target])
        .unwrap()
}

pub(super) fn profile_struct_value() -> StructAspectValue {
    StructAspectValue::new([
        (
            FieldKey::new("name").unwrap(),
            AspectValue::String("Ada".into()),
        ),
        (FieldKey::new("rank").unwrap(), AspectValue::UInt32(7)),
    ])
    .unwrap()
}

fn retained_live_snapshot_identity(label: &str) -> WorthQuerySnapshotIdentity {
    WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(
            retained_live_fixture_position("snapshot", label),
            retained_live_fixture_position("snapshot-version", label),
        ),
    )
}

fn projection_values(
    values: impl IntoIterator<Item = (&'static str, AspectValue)>,
) -> BTreeMap<CanonicalFieldPath, AspectValue> {
    values
        .into_iter()
        .map(|(path, value)| (canonical_field_path(path), value))
        .collect()
}

fn retained_materialized_row(
    values: impl IntoIterator<Item = (&'static str, AspectValue)>,
) -> WorthQueryRetainedMaterializedRow {
    let values = values
        .into_iter()
        .map(|(path, value)| (retained_field_path(path), value))
        .collect();
    WorthQueryRetainedMaterializedRow::from_scalar_values(values)
        .expect("test retained materialized row should admit")
}

fn text_value(value: impl Into<String>) -> AspectValue {
    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value)
}

fn retained_field_path(path: &str) -> WorthQueryRetainedFieldPath {
    WorthQueryRetainedFieldPath::from_canonical_field_path(canonical_field_path(path))
}

fn canonical_field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.')
            .map(|segment| FieldKey::new(segment.to_string()))
            .collect::<Option<Vec<_>>>()
            .expect("test field path should be canonical"),
    )
    .expect("test field path should not be empty")
}

fn retained_live_fixture_position(namespace: &str, evidence: &str) -> u64 {
    let mut acc = 14_695_981_039_346_656_037_u64;
    for byte in namespace.bytes().chain(evidence.bytes()) {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1_099_511_628_211_u64);
    }
    acc
}

pub(super) fn request_for_kind(kind: ProjectionFactKind) -> ProjectMaterializedFacts {
    match kind {
        ProjectionFactKind::EntityIdentity => {
            ProjectMaterializedFacts::declare().entity_identities()
        }
        ProjectionFactKind::ViewLocalIdentity => {
            ProjectMaterializedFacts::declare().view_local_identities()
        }
        ProjectionFactKind::TargetIdentity => ProjectMaterializedFacts::declare().target_identity(),
        ProjectionFactKind::SourceReference => {
            ProjectMaterializedFacts::declare().source_references()
        }
        ProjectionFactKind::EffectContinuity => {
            ProjectMaterializedFacts::declare().effect_continuity_facts()
        }
        ProjectionFactKind::Membership => ProjectMaterializedFacts::declare().memberships(),
        ProjectionFactKind::RelationEndpoint => {
            ProjectMaterializedFacts::declare().relation_endpoints()
        }
        ProjectionFactKind::DisplayField => ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
        ProjectionFactKind::DerivedField => ProjectMaterializedFacts::declare().derived_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    }
}

pub(super) fn visible_fields_for_kind(kind: ProjectionFactKind) -> Vec<&'static str> {
    match kind {
        ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
            vec!["profile.display_name"]
        }
        _ => vec!["identity.id"],
    }
}
