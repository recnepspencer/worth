use std::collections::{BTreeMap, BTreeSet};

use super::fixtures::{local_identity_collection_read, local_identity_read};
use crate::authoring::{
    AspectFieldSelector, AspectName, AuthoredResultShapeField, EqualityPredicate, FieldName,
    WorthQueryPredicateOperand,
};
use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQueryWorkspaceError,
};
use crate::ordinary::count::declare_count;
use crate::ordinary::read::{admit_read_context_declaration, current};
use crate::runtime::tests::support::complete_backend_from_parts_builder;
use crate::runtime::{
    WorthQueryLiveArtifactTarget, WorthQueryMutationReceipt, WorthQueryReadBuilder,
    WorthQueryReadDenialKind, WorthQueryReadFamily, WorthQueryRuntime,
    WorthQueryRuntimeSourceAdapter,
};
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};

#[test]
fn ordinary_count_materializes_real_collection_rows_and_exact_work_evidence() {
    let mut workspace = count_runtime(["ada", "grace", "linus"])
        .workspace("ordinary-count-real-rows")
        .expect("count workspace should open");
    let result = declare_count(local_identity_collection_read)
        .expect("collection count should declare")
        .using(current())
        .run(&mut workspace)
        .into_result()
        .expect("collection count should execute")
        .into_result();

    assert_eq!(result.count(), 3);
    assert_eq!(
        result.receipt().collection_result_family(),
        Some(&crate::collection::CollectionResultFamily::CountAggregate)
    );
    assert_eq!(
        result
            .receipt()
            .breadth()
            .execution_records_examined_count(),
        3
    );
    assert_eq!(
        result.receipt().breadth().execution_records_emitted_count(),
        1
    );
    assert_eq!(
        result.receipt().breadth().execution_aggregate_input_count(),
        3
    );
    assert_eq!(result.receipt().breadth().execution_rollup_input_count(), 3);
}

#[test]
fn ordinary_count_preserves_zero_for_an_empty_authoritative_collection() {
    let mut workspace = count_runtime(std::iter::empty::<&str>())
        .workspace("ordinary-count-empty")
        .expect("empty count workspace should open");
    let result = declare_count(local_identity_collection_read)
        .expect("empty collection count should declare")
        .using(current())
        .run(&mut workspace)
        .into_result()
        .expect("empty collection count should execute")
        .into_result();

    assert_eq!(result.count(), 0);
    assert_eq!(
        result.receipt().breadth().execution_aggregate_input_count(),
        0
    );
    assert_eq!(
        result
            .receipt()
            .breadth()
            .execution_records_examined_count(),
        0
    );
}

#[test]
fn ordinary_count_applies_declared_predicates_before_aggregation() {
    let mut workspace = count_runtime_from_rows(vec![
        profile_row("ada", "Ada Lovelace"),
        profile_row("grace", "Grace Hopper"),
        profile_row("adam", "Adam Smith"),
    ])
    .workspace("ordinary-count-filtered")
    .expect("filtered count workspace should open");
    let result = declare_count(|read| {
        read.explicit_broad_search_collection(
            "user",
            profile_schema(),
            |query| {
                query
                    .where_equal(
                        EqualityPredicate::new(
                            "profile",
                            "display_name",
                            WorthQueryPredicateOperand::string("Ada Lovelace".to_string()),
                        )
                        .expect("equality predicate should build"),
                    )
                    .project(
                        AspectFieldSelector::new("identity", "id")
                            .expect("identity projection should build"),
                    )
            },
            |shape| {
                shape.field(
                    AuthoredResultShapeField::new("identity", "id", "id")
                        .expect("identity result field should build"),
                )
            },
        )
    })
    .expect("filtered count should declare")
    .using(current())
    .run(&mut workspace)
    .into_result()
    .expect("filtered count should execute")
    .into_result();

    assert_eq!(result.count(), 1);
    assert_eq!(
        result
            .receipt()
            .breadth()
            .execution_records_examined_count(),
        3
    );
    assert_eq!(
        result.receipt().breadth().execution_aggregate_input_count(),
        1
    );
}

#[test]
fn ordinary_count_matches_the_internal_admitted_phase_chain_exactly() {
    let mut workspace = count_runtime(["ada", "grace", "linus", "margaret"])
        .workspace("ordinary-count-parity")
        .expect("count parity workspace should open");
    let ordinary = declare_count(local_identity_collection_read)
        .expect("ordinary count should declare")
        .using(current())
        .run(&mut workspace)
        .into_result()
        .expect("ordinary count should execute")
        .into_result();

    let oracle_intent = local_identity_collection_read(WorthQueryReadBuilder::declaration())
        .expect("oracle collection query should declare");
    let admitted_context = admit_read_context_declaration(&oracle_intent, current().into())
        .expect("oracle context should admit");
    let (authority, planning_authority, _) = admitted_context.into_parts();
    let oracle_graph = oracle_intent
        .plan_count(planning_authority)
        .expect("oracle count should plan");
    let oracle_plan_digest = oracle_graph
        .execution_plan()
        .query()
        .plan_digest()
        .evidence_identity();
    let oracle_family = WorthQueryReadFamily::new_kernel_only("declared_count", oracle_graph);
    let oracle = workspace
        .read_family_intent_in_graph_read_authority(&oracle_family, &authority)
        .admit()
        .expect("oracle count should admit")
        .execute_count()
        .expect("oracle count should execute");

    assert_eq!(ordinary.count(), oracle.count());
    assert_eq!(
        ordinary.receipt().execution_plan_evidence_identity(),
        &oracle_plan_digest
    );
    assert_eq!(ordinary.receipt(), oracle.receipt());
    assert_eq!(ordinary, oracle);
}

#[test]
fn detail_shape_is_denied_before_count_planning_or_runtime_contact() {
    let stop = declare_count(local_identity_read)
        .expect_err("detail reads cannot mint count aggregate declarations");

    assert_eq!(
        stop.denial().kind(),
        &WorthQueryReadDenialKind::AuthoringDenied
    );
    assert_eq!(
        stop.next_action(),
        crate::ordinary::read::WorthQueryReadNextAction::ReviseDeclaration
    );
}

fn count_runtime(labels: impl IntoIterator<Item = &'static str>) -> WorthQueryRuntime {
    let rows = labels.into_iter().map(identity_row).collect();
    count_runtime_from_rows(rows)
}

fn count_runtime_from_rows(rows: Vec<WorthQueryEntity>) -> WorthQueryRuntime {
    complete_backend_from_parts_builder()
        .source_adapter(CountCollectionSourceAdapter {
            rows,
            declared_targets: BTreeSet::new(),
        })
        .build_backend_from_parts()
        .build()
        .expect("complete count runtime should build")
}

struct CountCollectionSourceAdapter {
    rows: Vec<WorthQueryEntity>,
    declared_targets: BTreeSet<WorthQueryLiveArtifactTarget>,
}

impl WorthQueryRuntimeSourceAdapter for CountCollectionSourceAdapter {
    fn declare_live_view(
        &mut self,
        name: String,
        _request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        self.declared_targets
            .insert(WorthQueryLiveArtifactTarget::from_view_name(name.clone()));
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        self.declared_targets
            .remove(&WorthQueryLiveArtifactTarget::from_view_name(name));
        Ok(())
    }

    fn live_entities_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, crate::memory_workspace::WorthQueryWorkspaceError> {
        Ok({
            if self.declared_targets.contains(target) {
                self.rows.clone()
            } else {
                Vec::new()
            }
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

fn identity_row(label: &'static str) -> WorthQueryEntity {
    WorthQueryEntity::from_native_field_values(
        crate::memory_workspace::admit_authored_entity_label(label),
        BTreeMap::from([(
            native_field_path("identity.id"),
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                label.to_string(),
            ),
        )]),
    )
}

fn profile_row(label: &'static str, display_name: &'static str) -> WorthQueryEntity {
    WorthQueryEntity::from_native_field_values(
        crate::memory_workspace::admit_authored_entity_label(label),
        BTreeMap::from([
            (
                native_field_path("identity.id"),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                    label.to_string(),
                ),
            ),
            (
                native_field_path("profile.display_name"),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                    display_name.to_string(),
                ),
            ),
        ]),
    )
}

fn profile_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "ordinary-count-profile",
        [
            SchemaFieldView::new(
                AspectName::new("identity").expect("identity aspect should build"),
                FieldName::new("id").expect("identity field should build"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                AspectName::new("profile").expect("profile aspect should build"),
                FieldName::new("display_name").expect("display-name field should build"),
                ScalarAspectType::String,
            ),
        ],
        [],
    )
}

fn native_field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.')
            .map(FieldKey::new)
            .collect::<Option<Vec<_>>>()
            .expect("test field path should be foundational"),
    )
    .expect("test field path should not be empty")
}
