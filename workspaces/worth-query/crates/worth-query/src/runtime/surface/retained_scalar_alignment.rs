use super::{
    WorthQueryDerivedArtifactBinding, WorthQueryDerivedMaterializationTarget,
    WorthQueryRetainedFieldPath, WorthQueryRetainedScalarFactSet,
};
use crate::identity::hash_parts;
use crate::runtime::computed::WorthQueryDerivedViewHandle;
use crate::runtime::WorthQueryRuntimeError;
use worth_foundational::facade::{prepare_aspect_value_identity_basis, AspectValue};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryRetainedScalarAlignmentFact {
    left_field_path: WorthQueryRetainedFieldPath,
    right_field_path: WorthQueryRetainedFieldPath,
    value: AspectValue,
}

impl WorthQueryRetainedScalarAlignmentFact {
    pub fn left_field_path(&self) -> &WorthQueryRetainedFieldPath {
        &self.left_field_path
    }

    pub fn right_field_path(&self) -> &WorthQueryRetainedFieldPath {
        &self.right_field_path
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryRetainedScalarAlignment {
    artifact_name: String,
    binding_digest: String,
    left_target: WorthQueryDerivedMaterializationTarget,
    right_target: WorthQueryDerivedMaterializationTarget,
    alignment_digest: String,
    aligned_facts: Vec<WorthQueryRetainedScalarAlignmentFact>,
}

impl WorthQueryRetainedScalarAlignment {
    pub fn artifact_name(&self) -> &str {
        &self.artifact_name
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    pub fn left_target(&self) -> &WorthQueryDerivedMaterializationTarget {
        &self.left_target
    }

    pub fn right_target(&self) -> &WorthQueryDerivedMaterializationTarget {
        &self.right_target
    }

    pub fn alignment_digest(&self) -> &str {
        &self.alignment_digest
    }

    pub fn aligned_field_count(&self) -> usize {
        self.aligned_facts.len()
    }

    pub fn aligned_facts(&self) -> &[WorthQueryRetainedScalarAlignmentFact] {
        &self.aligned_facts
    }
}

impl WorthQueryDerivedArtifactBinding {
    pub fn verify_scalar_alignment<V1, V2, I>(
        &self,
        left_view: &WorthQueryDerivedViewHandle<V1>,
        right_view: &WorthQueryDerivedViewHandle<V2>,
        field_pairs: I,
    ) -> Result<WorthQueryRetainedScalarAlignment, WorthQueryRuntimeError>
    where
        I: IntoIterator<Item = (WorthQueryRetainedFieldPath, WorthQueryRetainedFieldPath)>,
    {
        let normalized_pairs = field_pairs.into_iter().collect::<Vec<_>>();
        let left_target = WorthQueryDerivedMaterializationTarget::from(left_view);
        let right_target = WorthQueryDerivedMaterializationTarget::from(right_view);
        let left_facts = self.consume_scalar_field_paths_for_target(
            &left_target,
            normalized_pairs
                .iter()
                .map(|(left_path, _)| left_path.clone())
                .collect(),
        )?;
        let right_facts = self.consume_scalar_field_paths_for_target(
            &right_target,
            normalized_pairs
                .iter()
                .map(|(_, right_path)| right_path.clone())
                .collect(),
        )?;
        verify_scalar_alignment_between_fact_sets(
            self,
            left_target,
            right_target,
            &left_facts,
            &right_facts,
            &normalized_pairs,
        )
    }
}

fn verify_scalar_alignment_between_fact_sets(
    binding: &WorthQueryDerivedArtifactBinding,
    left_target: WorthQueryDerivedMaterializationTarget,
    right_target: WorthQueryDerivedMaterializationTarget,
    left_facts: &WorthQueryRetainedScalarFactSet,
    right_facts: &WorthQueryRetainedScalarFactSet,
    field_pairs: &[(WorthQueryRetainedFieldPath, WorthQueryRetainedFieldPath)],
) -> Result<WorthQueryRetainedScalarAlignment, WorthQueryRuntimeError> {
    let left_view_name = left_target.view_name();
    let right_view_name = right_target.view_name();
    let aligned_facts = field_pairs
        .iter()
        .map(|(left_field_path, right_field_path)| {
            let left_field_key = left_field_path.terminal_projection_for_boundary();
            let right_field_key = right_field_path.terminal_projection_for_boundary();
            let left_value =
                left_facts
                    .field_value_at(left_field_path)
                    .ok_or_else(|| WorthQueryRuntimeError::RetainedRowDecode {
                        view_name: left_view_name.to_string(),
                        stage: "retained-scalar-alignment",
                        message: format!(
                            "retained scalar fact set for `{left_view_name}` omitted `{left_field_key}`"
                        ),
                    })?;
            let right_value =
                right_facts
                    .field_value_at(right_field_path)
                    .ok_or_else(|| WorthQueryRuntimeError::RetainedRowDecode {
                        view_name: right_view_name.to_string(),
                        stage: "retained-scalar-alignment",
                        message: format!(
                            "retained scalar fact set for `{right_view_name}` omitted `{right_field_key}`"
                        ),
                    })?;
            if left_value != right_value {
                return Err(WorthQueryRuntimeError::RetainedRowDecode {
                    view_name: binding.artifact_name().to_string(),
                    stage: "retained-scalar-alignment",
                    message: format!(
                        "retained scalar alignment diverged between `{left_view_name}.{left_field_key}` and `{right_view_name}.{right_field_key}`"
                    ),
                });
            }
            Ok(WorthQueryRetainedScalarAlignmentFact {
                left_field_path: left_field_path.clone(),
                right_field_path: right_field_path.clone(),
                value: left_value.clone(),
            })
        })
        .collect::<Result<Vec<_>, WorthQueryRuntimeError>>()?;

    let alignment_digest = hash_parts(
        &std::iter::once("worth_query_retained_scalar_alignment_v1".to_string())
            .chain(std::iter::once(format!(
                "artifact:{}",
                binding.artifact_name()
            )))
            .chain(std::iter::once(format!(
                "binding:{}",
                binding.binding_for_reporting()
            )))
            .chain(std::iter::once(format!("left:{left_view_name}")))
            .chain(std::iter::once(format!("right:{right_view_name}")))
            .chain(aligned_facts.iter().map(|fact| {
                format!(
                    "pair:{}={}:{}",
                    fact.left_field_path().terminal_projection_for_boundary(),
                    fact.right_field_path().terminal_projection_for_boundary(),
                    prepare_aspect_value_identity_basis(fact.value()).as_str()
                )
            }))
            .collect::<Vec<_>>(),
    );

    Ok(WorthQueryRetainedScalarAlignment {
        artifact_name: binding.artifact_name().to_string(),
        binding_digest: binding.binding_for_reporting().to_string(),
        left_target,
        right_target,
        alignment_digest,
        aligned_facts,
    })
}

#[cfg(test)]
mod tests {
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

    fn binding(
        row: WorthQueryRetainedMaterializedRow,
    ) -> Result<WorthQueryDerivedArtifactBinding, IdentityDerivationDenial> {
        let snapshot_identity =
            crate::memory_workspace::admit_external_snapshot_label("snapshot:test")?;
        let materialization = WorthQueryDerivedMaterializationResult::from_retained_rows(
            vec![row],
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
    fn retained_scalar_alignment_verifies_mapped_fields() -> Result<(), IdentityDerivationDenial> {
        let left_view = view_handle();
        let right_view = view_handle();
        let alignment = binding(retained_row([
            ("authority_snapshot_id", AspectValue::Int64(7)),
            (
                "nested.truth_basis_digest_hex",
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("basis:test"),
            ),
        ]))?
        .verify_scalar_alignment(
            &left_view,
            &right_view,
            [
                (
                    retained_field_path("authority_snapshot_id").expect("left path admits"),
                    retained_field_path("authority_snapshot_id").expect("right path admits"),
                ),
                (
                    retained_field_path("nested.truth_basis_digest_hex").expect("left path admits"),
                    retained_field_path("nested.truth_basis_digest_hex")
                        .expect("right path admits"),
                ),
            ],
        )
        .expect("alignment should succeed");

        assert_eq!(alignment.artifact_name(), "artifact:test");
        assert_eq!(alignment.aligned_field_count(), 2);
        assert!(!alignment.alignment_digest().is_empty());

        Ok(())
    }

    #[test]
    fn retained_scalar_alignment_rejects_divergent_value() -> Result<(), IdentityDerivationDenial> {
        let left_view = view_handle();
        let right_view = view_handle();
        let error = binding(retained_row([
            ("left", AspectValue::Int64(1)),
            ("right", AspectValue::Int64(2)),
        ]))?
        .verify_scalar_alignment(
            &left_view,
            &right_view,
            [(
                retained_field_path("left").expect("left path admits"),
                retained_field_path("right").expect("right path admits"),
            )],
        )
        .expect_err("divergent scalar fields should fail");

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
}
