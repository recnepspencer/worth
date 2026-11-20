# Schema Validation

Pointers to canonical query validation and to the Native boundary entry points
for schema changes and reads. For laws enforced when an application changes its
graph, see [Relation laws](relation-laws.md).

## Canonical query validation

- `worth_query_decl::facade::validation::validate_canonical_bundle` ([source](../../crates/worth-query-declaration/src/validation/pipeline.rs#L19))
- `worth_query_decl::facade::validation::ValidatedQueryBundle` ([source](../../crates/worth-query-declaration/src/validation/artifacts/bundle.rs#L9))
- `worth_query_decl::facade::validation::QueryValidationReport` ([source](../../crates/worth-query-declaration/src/validation/report.rs#L107))
- `worth_query_decl::facade::validation::ValidationEvent` ([source](../../crates/worth-query-declaration/src/validation/report.rs#L2))
- `worth_query_decl::facade::validation::QueryValidationError` ([source](../../crates/worth-query-declaration/src/validation/errors.rs#L15))
- `worth_query_decl::facade::validation::ValidationFailureClass` ([source](../../crates/worth-query-declaration/src/validation/errors.rs#L2))
- `worth_query_decl::facade::canonicalization::CanonicalQueryBundle` ([source](../../crates/worth-query-declaration/src/canonicalization/bundle_state.rs#L10))
- `worth_query_decl::facade::typed::QuerySchemaView` ([source](../../crates/worth-query-declaration/src/schema_view/mod.rs#L15))

## Candidate Structural Source Changes

- `worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner::prepare_derived_relation_changes` ([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/application_invariant/typed_view/derived_changes.rs#L45))
- `worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantContext::derived_relation_changes` ([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/application_invariant/typed_view/derived_changes.rs#L66))
- `worth_query_host::facade::application_invariants::WorthQueryPreparedInvariantDerivedChanges` ([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/application_invariant/typed_view/derived_changes.rs#L19))
- `worth_query_host::facade::application_invariants::WorthQueryInvariantDerivedChanges` ([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/application_invariant/typed_view/derived_changes.rs#L28))
- Guide: [Prepare candidate structural source changes](../../../../docs/build-an-application.md#prepare-candidate-structural-source-changes)

## Explicit row-local exact managed reads

- `worth_query_decl::facade::application_query::ApplicationQueryDefinitionBuilder::managed_index_refined` ([source](../../crates/worth-query-declaration/src/application_query/definition/builder/managed_index.rs#L27))
- `worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantContext::refined_managed_index` ([source](../../crates/worth-query-execution/src/domain_computation/primary_graph/application_invariant/typed_view/managed_index.rs#L97))
- Guide: [Construct committed provider indexes](../../../../docs/build-an-application.md#construct-committed-provider-indexes)

## Protected ordinary child continuation

- `worth_query_host::facade::application_entry::WorthQueryApplicationQueryRequest::page_governed` ([source](../../crates/worth-query-publication/src/application_entry/query/governed_continuation.rs#L74))
- `worth_query_host::facade::application_entry::WorthQueryApplicationQueryRequest::resume_governed` ([source](../../crates/worth-query-publication/src/application_entry/query/governed_continuation.rs#L122))
- Guide: [Protected continuation pages](../../../../docs/build-an-application.md#protected-continuation-pages)
