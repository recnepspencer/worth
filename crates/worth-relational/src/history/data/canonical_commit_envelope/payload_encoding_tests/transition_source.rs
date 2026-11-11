//! A genuine Native transition supplies the complete Original semantic image.
use crate::history::data::SharedCanonicalCommitEnvelope;
use crate::schema::data::{
    ProposedSchemaTransition, RelationalSchemaRegistry, SchemaReconciliationPolicy,
};
use crate::tests::support::*;

pub(super) fn commit(
    registry: &RelationalSchemaRegistry,
    proposed: ProposedSchemaTransition,
    policy: Option<SchemaReconciliationPolicy>,
) -> SharedCanonicalCommitEnvelope {
    let original = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&original);
    let mut runtime = runtime_with_declared_aspect_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::CertificationCore,
        crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let first = create_entity_outcome(&execution, &runtime, "codec-predecessor");
    release_test_commit_snapshot(&runtime, &first);
    drop(first);
    runtime.set_schema_registry_for_test(registry.clone());
    let options = test_owner_transaction_validation_input_for_main(&runtime)
        .with_schema_transition(proposed, policy);
    let mut transaction = runtime
        .begin_branch_transaction(options.basis(), options.intent().clone())
        .unwrap();
    transaction
        .push_batch(
            batch_create("codec-transition"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let result = transaction.commit(&execution, &runtime).unwrap();
    let envelope = result.publication().envelope.clone();
    release_test_commit_snapshot(&runtime, &result);
    envelope
}
