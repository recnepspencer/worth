mod authentication;
mod public_merge;
mod runtime_world;
mod schema;

use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationPrincipalKey;

use self::authentication::external_identity;
use self::runtime_world::{
    host_relational_runtime, mixed_basis_relational_runtime, CommittingWriteAuthority,
};
use self::schema::{
    primary_graph_domain_package, IdentityBinding, PrimaryGraphCompositionSchema, Principal,
};
use super::support::{
    complete_backend_from_parts_builder, custom_backend_without_primary_graph_transfer_builder,
    insert_command, test_string_aspect_value,
};

use crate::runtime::WorthQueryConditionalExecutionResources;

#[test]
fn ordinary_write_and_product_branch_creation_share_one_configured_relational_graph() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let subject = "dynamic-user-7f643b";
    let mut runtime = complete_backend_from_parts_builder()
        .conditional_execution_resources(WorthQueryConditionalExecutionResources::development())
        .domain_package(primary_graph_domain_package())
        .expect("primary graph domain package should admit")
        .relational_runtime(host_relational_runtime())
        .write_authority(CommittingWriteAuthority)
        .application_primary_graph::<PrimaryGraphCompositionSchema, _>(move |graph| {
            graph.bind_principal(
                IdentityBinding::reference(),
                WorthQueryApplicationPrincipalKey::<PrimaryGraphCompositionSchema, Principal>::new(
                    "principal-dynamic-user-7f643b",
                )
                .expect("typed principal key should admit"),
                7_u64,
                external_identity(subject),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        })
        .expect("one typed primary graph should configure")
        .build_backend_from_parts()
        .build(request_execution)
        .expect("the configured runtime should publish");

    let publication = runtime
        .primary_graph_publication()
        .expect("primary graph publication evidence should be retained");
    assert_eq!(publication.principal_binding_count(), 1);
    assert_eq!(publication.identity_index_count(), 1);

    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [(
                    "identity.id",
                    test_string_aspect_value("ordinary-shared-root-write"),
                )],
            ),
        )
        .expect("ordinary Query write should commit through the shared graph");

    let workspace = runtime
        .workspace("shared-primary-graph")
        .expect("the installed runtime must expose its Product World");
    let root = workspace.current_world();
    let sibling = workspace
        .runtime
        .installed_product
        .world
        .with_advancement(|phase| {
            workspace
                .branches()
                .fork(root)
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
        })
        .expect("the host scope opens")
        .expect("the Product World should create from the exact primary graph source");
    assert!(sibling.occurrence_ordinal() > root.occurrence_ordinal());
}

pub(super) fn primary_graph_merge_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::runtime::WorthQueryRuntime {
    complete_backend_from_parts_builder()
        .conditional_execution_resources(WorthQueryConditionalExecutionResources::development())
        .domain_package(primary_graph_domain_package())
        .expect("primary graph domain package admits")
        .relational_runtime(host_relational_runtime())
        .write_authority(CommittingWriteAuthority)
        .application_primary_graph::<PrimaryGraphCompositionSchema, _>(|graph| {
            graph.bind_principal(
                IdentityBinding::reference(),
                WorthQueryApplicationPrincipalKey::<PrimaryGraphCompositionSchema, Principal>::new(
                    "principal-settlement-owner",
                )
                .expect("typed principal key admits"),
                11_u64,
                external_identity("settlement-owner"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        })
        .expect("primary graph configures")
        .build_backend_from_parts()
        .build(request_execution)
        .expect("post-installation bridge runtime publishes")
}

#[test]
fn mixed_host_schema_basis_is_rejected_before_primary_graph_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let result = complete_backend_from_parts_builder()
        .domain_package(primary_graph_domain_package())
        .expect("primary graph domain package should admit")
        .relational_runtime(mixed_basis_relational_runtime())
        .application_primary_graph::<PrimaryGraphCompositionSchema, _>(|graph| {
            graph.bind_principal(
                IdentityBinding::reference(),
                WorthQueryApplicationPrincipalKey::<PrimaryGraphCompositionSchema, Principal>::new(
                    "principal-hostile-basis",
                )
                .expect("typed principal key should admit"),
                1_u64,
                external_identity("hostile-basis-subject"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        })
        .expect("typed primary graph contribution should configure")
        .build_backend_from_parts()
        .build(request_execution);
    let error = match result {
        Ok(_) => panic!("mixed Relational schema authority must not publish a primary graph"),
        Err(error) => error,
    };

    match error {
        crate::runtime::WorthQueryRuntimeError::InvariantRegistration { stage, message } => {
            assert_eq!(stage, "primary_graph_bootstrap_preparation");
            assert!(message.contains("RelationalSchemaRejected"));
            assert!(message.contains("mixed schema basis"));
        }
        other => panic!("unexpected mixed-basis denial: {other:?}"),
    }
}

#[test]
fn custom_backend_must_explicitly_implement_primary_graph_transfer() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let result = custom_backend_without_primary_graph_transfer_builder()
        .domain_package(primary_graph_domain_package())
        .expect("primary graph domain package should admit")
        .application_primary_graph::<PrimaryGraphCompositionSchema, _>(|graph| {
            graph.bind_principal(
                IdentityBinding::reference(),
                WorthQueryApplicationPrincipalKey::<PrimaryGraphCompositionSchema, Principal>::new(
                    "principal-custom-backend",
                )
                .expect("typed principal key should admit"),
                1_u64,
                external_identity("custom-backend-subject"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        })
        .expect("typed primary graph contribution should configure")
        .build(request_execution);
    let error = match result {
        Ok(_) => panic!("an implicit custom-backend primary graph must not publish"),
        Err(error) => error,
    };

    match error {
        crate::runtime::WorthQueryRuntimeError::Workspace(denial) => {
            assert!(denial
                .to_string()
                .contains("cannot surrender an unpublished primary graph runtime"));
        }
        other => panic!("unexpected custom-backend transfer denial: {other:?}"),
    }
}
