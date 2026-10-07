use std::collections::BTreeMap;

use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQueryWorkspaceError,
};
use crate::runtime::tests::support::complete_backend_from_parts_builder;
use crate::runtime::{
    WorthQueryLiveArtifactTarget, WorthQueryMutationReceipt, WorthQueryRuntime,
    WorthQueryRuntimeSourceAdapter,
};
use crate::schema_view::QuerySchemaView;
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};

pub(super) fn read_runtime_with_permissive_policy_row() -> WorthQueryRuntime {
    complete_backend_from_parts_builder()
        .source_adapter(PermissivePolicyRowSourceAdapter)
        .build_backend_from_parts()
        .build()
        .expect("complete permissive-row runtime should build")
}

struct PermissivePolicyRowSourceAdapter;

impl WorthQueryRuntimeSourceAdapter for PermissivePolicyRowSourceAdapter {
    fn declare_live_view(
        &mut self,
        name: String,
        _request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, _name: &str) -> Result<(), WorthQueryWorkspaceError> {
        Ok(())
    }

    fn live_entities_for_target(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, crate::memory_workspace::WorthQueryWorkspaceError> {
        Ok({
            vec![WorthQueryEntity::from_native_field_values(
                crate::memory_workspace::admit_authored_entity_label("user"),
                BTreeMap::from([
                    (native_field_path("identity", "id"), string_value("user")),
                    (
                        native_field_path("profile", "display_name"),
                        string_value("Ada"),
                    ),
                    (native_field_path("profile", "handle"), string_value("@ada")),
                ]),
            )]
        })
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        _receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        Vec::new()
    }
}

fn native_field_path(aspect: &str, field: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new([
        FieldKey::new(aspect).expect("test aspect must be a field-path segment"),
        FieldKey::new(field).expect("test field must be a field-path segment"),
    ])
    .expect("test field path must not be empty")
}

fn string_value(value: &str) -> worth_foundational::facade::AspectValue {
    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value.to_string())
}
