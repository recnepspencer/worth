use super::{
    WorthQueryDerivedMaterializationBundle, WorthQueryDerivedMaterializationResult,
    WorthQueryDerivedMaterializationTarget,
};
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::computed::WorthQueryDerivedViewHandle;
use crate::runtime::WorthQueryRuntimeError;

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDerivedArtifactBinding {
    artifact_name: String,
    binding_identity: WorthQueryEvidenceIdentity,
    bundle: WorthQueryDerivedMaterializationBundle,
    targets: Vec<WorthQueryDerivedMaterializationTarget>,
}

impl WorthQueryDerivedArtifactBinding {
    pub(in crate::runtime) fn bind(
        bundle: WorthQueryDerivedMaterializationBundle,
        artifact_name: impl Into<String>,
        required_targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let artifact_name = artifact_name.into();
        Self::bind_inner(bundle, artifact_name, required_targets)
    }

    pub(in crate::runtime) fn bind_with_identity(
        bundle: WorthQueryDerivedMaterializationBundle,
        artifact_identity: WorthQueryEvidenceIdentity,
        required_targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<Self, WorthQueryRuntimeError> {
        Self::bind_inner(
            bundle,
            artifact_identity.as_str().to_string(),
            required_targets,
        )
    }

    fn bind_inner(
        bundle: WorthQueryDerivedMaterializationBundle,
        artifact_name: String,
        required_targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let mut targets = required_targets.into_iter().collect::<Vec<_>>();
        targets.sort();
        targets.dedup();

        if bundle.target_count() != targets.len()
            || !targets.iter().all(|target| bundle.includes_target(target))
        {
            let target_view_names = terminal_target_view_names(&targets);
            let bundle_view_names = bundle
                .terminal_target_view_names_projection()
                .collect::<Vec<_>>();
            return Err(WorthQueryRuntimeError::RetainedRowDecode {
                view_name: artifact_name.clone(),
                stage: "derived-artifact-binding",
                message: format!(
                    "retained artifact binding `{artifact_name}` requires exact target set {:?}, but bundle carried {:?}",
                    target_view_names, bundle_view_names
                ),
            });
        }

        let binding_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::SharedReadGeneration)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_derived_artifact_binding_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("artifact"), &artifact_name)
                .field_shape(WorthQueryEvidenceTag::new("bundle"), bundle.bundle_digest())
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("target"),
                    targets
                        .iter()
                        .map(WorthQueryDerivedMaterializationTarget::terminal_view_name_projection),
                )
                .seal()?;

        Ok(Self {
            artifact_name,
            binding_identity,
            bundle,
            targets,
        })
    }

    pub fn artifact_name(&self) -> &str {
        &self.artifact_name
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub fn binding_for_reporting(&self) -> &str {
        self.binding_identity.as_str()
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        self.bundle.snapshot_identity()
    }

    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    pub fn targets(&self) -> &[WorthQueryDerivedMaterializationTarget] {
        &self.targets
    }

    pub fn terminal_target_view_names_projection(&self) -> impl Iterator<Item = &str> {
        self.targets.iter().map(|target| target.view_name())
    }

    pub fn materialization<T>(
        &self,
        view: &WorthQueryDerivedViewHandle<T>,
    ) -> Result<&WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        self.bundle.materialization(view)
    }

    pub(crate) fn materialization_for_target(
        &self,
        target: &WorthQueryDerivedMaterializationTarget,
    ) -> Result<&WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        self.bundle.materialization_for_target(target)
    }

    pub fn into_bundle(self) -> WorthQueryDerivedMaterializationBundle {
        self.bundle
    }
}

fn terminal_target_view_names(targets: &[WorthQueryDerivedMaterializationTarget]) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.terminal_view_name_projection().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
    use std::collections::BTreeMap;

    use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey, InternedString};

    use super::WorthQueryDerivedArtifactBinding;

    use crate::runtime::computed::WorthQueryDerivedViewHandle;
    use crate::runtime::surface::{
        WorthQueryDerivedMaterializationBundle, WorthQueryDerivedMaterializationReceipt,
        WorthQueryDerivedMaterializationResult, WorthQueryDerivedMaterializationTarget,
        WorthQueryRetainedFieldPath, WorthQueryRetainedMaterializedRow,
    };
    use crate::runtime::WorthQueryUnrefinedLiveShape;

    fn scalar_row(field: &str, value: AspectValue) -> WorthQueryRetainedMaterializedRow {
        WorthQueryRetainedMaterializedRow::from_scalar_values(BTreeMap::from([(
            retained_field_path(field).expect("field path admits"),
            value,
        )]))
        .expect("retained row should build")
    }

    fn retained_row(
        view_name: &str,
        value: &str,
    ) -> Result<WorthQueryDerivedMaterializationResult, IdentityDerivationDenial> {
        Ok(WorthQueryDerivedMaterializationResult::from_retained_rows(
            vec![scalar_row(
                "value",
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value),
            )],
            WorthQueryDerivedMaterializationReceipt::test_only(
                view_name,
                crate::memory_workspace::admit_external_snapshot_label("snapshot-test")?,
                format!("{view_name}-digest"),
            ),
        ))
    }

    fn binding() -> Result<
        (
            WorthQueryDerivedArtifactBinding,
            WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape>,
            WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape>,
            WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape>,
        ),
        IdentityDerivationDenial,
    > {
        let first = WorthQueryDerivedViewHandle::new("derived.first");
        let second = WorthQueryDerivedViewHandle::new("derived.second");
        let third = WorthQueryDerivedViewHandle::new("derived.third");
        let bundle = WorthQueryDerivedMaterializationBundle::new(
            crate::memory_workspace::admit_external_snapshot_label("snapshot-test")?,
            BTreeMap::from([
                (
                    WorthQueryDerivedMaterializationTarget::from(&first),
                    retained_row(first.name(), "first")?,
                ),
                (
                    WorthQueryDerivedMaterializationTarget::from(&second),
                    retained_row(second.name(), "second")?,
                ),
                (
                    WorthQueryDerivedMaterializationTarget::from(&third),
                    retained_row(third.name(), "third")?,
                ),
            ]),
        )?;
        let binding = WorthQueryDerivedArtifactBinding::bind(
            bundle,
            "test.binding",
            [
                WorthQueryDerivedMaterializationTarget::from(&first),
                WorthQueryDerivedMaterializationTarget::from(&second),
                WorthQueryDerivedMaterializationTarget::from(&third),
            ],
        )
        .expect("binding should succeed");
        Ok((binding, first, second, third))
    }

    fn retained_string_value(
        binding: &WorthQueryDerivedArtifactBinding,
        view: &WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape>,
    ) -> String {
        let value_path = retained_field_path("value").expect("value path admits");
        let value = binding
            .materialization(view)
            .expect("bound materialization should exist")
            .single_retained_row()
            .expect("bound materialization should carry one row")
            .scalar_value_at(&value_path)
            .expect("bound materialization should carry value field");
        let Some(value) = (value).raw_text() else {
            panic!("expected retained string value, got {value:?}");
        };
        value.to_owned()
    }

    #[test]
    fn row_pair_uses_bound_retained_artifact_identity() -> Result<(), IdentityDerivationDenial> {
        let (binding, first, second, _) = binding()?;

        assert_eq!(retained_string_value(&binding, &first), "first");
        assert_eq!(retained_string_value(&binding, &second), "second");

        Ok(())
    }

    #[test]
    fn row_triple_uses_bound_retained_artifact_identity() -> Result<(), IdentityDerivationDenial> {
        let (binding, first, second, third) = binding()?;

        assert_eq!(retained_string_value(&binding, &first), "first");
        assert_eq!(retained_string_value(&binding, &second), "second");
        assert_eq!(retained_string_value(&binding, &third), "third");

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
