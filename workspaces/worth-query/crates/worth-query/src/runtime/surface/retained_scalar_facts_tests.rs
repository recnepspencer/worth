use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};

use crate::runtime::surface::{
    WorthQueryDerivedArtifactBinding, WorthQueryDerivedMaterializationBundle,
    WorthQueryDerivedMaterializationReceipt, WorthQueryDerivedMaterializationResult,
    WorthQueryDerivedMaterializationTarget, WorthQueryRetainedFieldPath,
    WorthQueryRetainedMaterializedRow,
};
use crate::runtime::WorthQueryDerivedViewHandle;

fn retained_row(
    fields: impl IntoIterator<Item = (&'static str, AspectValue)>,
) -> WorthQueryRetainedMaterializedRow {
    let fields = fields
        .into_iter()
        .map(|(path, value)| {
            (
                retained_field_path(path).expect("retained field path admits"),
                value,
            )
        })
        .collect::<BTreeMap<_, _>>();
    WorthQueryRetainedMaterializedRow::from_scalar_values(fields)
        .expect("retained row should build")
}

fn binding() -> Result<WorthQueryDerivedArtifactBinding, IdentityDerivationDenial> {
    let snapshot_identity = crate::memory_workspace::admit_external_snapshot_label("snapshot:test");
    let materialization = WorthQueryDerivedMaterializationResult::from_retained_rows(
        vec![retained_row([
            ("authority_snapshot_id", AspectValue::Int64(7)),
            (
                "nested.truth_basis_digest_hex",
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("basis:test"),
            ),
        ])],
        WorthQueryDerivedMaterializationReceipt::test_only(
            "surface:test",
            snapshot_identity.clone(),
            "result:test",
        ),
    );
    let target = WorthQueryDerivedMaterializationTarget::new("surface:test");
    let bundle = WorthQueryDerivedMaterializationBundle::new(
        snapshot_identity,
        BTreeMap::from([(target.clone(), materialization)]),
    )?;
    Ok(
        WorthQueryDerivedArtifactBinding::bind(bundle, "artifact:test", [target])
            .expect("binding should build"),
    )
}

fn view_handle() -> WorthQueryDerivedViewHandle<crate::runtime::WorthQueryUnrefinedLiveShape> {
    WorthQueryDerivedViewHandle::new("surface:test")
}

#[test]
fn retained_scalar_fact_set_reads_nested_fields() -> Result<(), IdentityDerivationDenial> {
    let view = view_handle();
    let truth_basis_digest =
        retained_field_path("nested.truth_basis_digest_hex").expect("field path should parse");
    let authority_snapshot_id =
        retained_field_path("authority_snapshot_id").expect("field path should parse");
    let facts = binding()?
        .consume_scalar_fields(
            &view,
            [truth_basis_digest.clone(), authority_snapshot_id.clone()],
        )
        .expect("scalar facts should extract");

    assert_eq!(facts.artifact_name(), "artifact:test");
    assert_eq!(facts.target().view_name(), "surface:test");
    assert_eq!(facts.field_count(), 2);
    assert_eq!(
        facts.field_value_at(&authority_snapshot_id),
        Some(&AspectValue::Int64(7))
    );
    assert_eq!(
        facts.field_value_at(&truth_basis_digest),
        Some(&crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("basis:test"))
    );
    assert!(!facts.fact_set_digest().is_empty());

    Ok(())
}

#[test]
fn retained_scalar_fact_set_rejects_missing_field() -> Result<(), IdentityDerivationDenial> {
    let view = view_handle();
    let missing_field = retained_field_path("missing.field").expect("field path should parse");
    let error = binding()?
        .consume_scalar_fields(&view, [missing_field])
        .expect_err("missing field should fail");

    assert!(matches!(
        error,
        crate::runtime::WorthQueryRuntimeError::RetainedRowDecode { .. }
    ));

    Ok(())
}

fn retained_field_path(path: &str) -> Result<WorthQueryRetainedFieldPath, String> {
    let fields = path
        .split('.')
        .map(|segment| {
            FieldKey::new(segment.to_string())
                .ok_or_else(|| format!("`{path}` is not a retained scalar field path"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let path = CanonicalFieldPath::new(fields)
        .ok_or_else(|| format!("`{path}` is not a retained scalar field path"))?;
    Ok(WorthQueryRetainedFieldPath::from_canonical_field_path(path))
}
