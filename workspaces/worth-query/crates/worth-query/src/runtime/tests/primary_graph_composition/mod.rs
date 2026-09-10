mod authentication;
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

use crate::ordinary::workflow::{branch_merge, declare_branch_merge};
use crate::runtime::WorthQueryConditionalExecutionResources;

#[test]
fn ordinary_write_and_product_branch_creation_share_one_configured_relational_graph() {
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
        .build()
        .expect("the configured runtime should publish");

    let publication = runtime
        .primary_graph_publication()
        .expect("primary graph publication evidence should be retained");
    assert_eq!(publication.principal_binding_count(), 1);
    assert_eq!(publication.identity_index_count(), 1);

    runtime
        .write(insert_command(
            "Task",
            [(
                "identity.id",
                test_string_aspect_value("ordinary-shared-root-write"),
            )],
        ))
        .expect("ordinary Query write should commit through the shared graph");

    let workspace = runtime
        .workspace("shared-primary-graph")
        .expect("the installed runtime must expose its Product World");
    let root = workspace.current_world();
    let sibling = workspace
        .branches()
        .fork(root)
        .components(|components| {
            components
                .reuse_exact_relational_basis()
                .reuse_exact_signal_basis()
        })
        .create()
        .expect("the Product World should create from the exact primary graph source");
    assert!(sibling.occurrence_ordinal() > root.occurrence_ordinal());
}

#[test]
fn post_installation_bridge_backend_repairs_settlement_through_its_public_owner() {
    let mut runtime = primary_graph_merge_runtime();
    prepare_post_installation_merge_fault(&mut runtime);
    let mut workspace = runtime
        .workspace("primary-graph-public-settlement")
        .expect("primary-graph workspace opens");
    let declaration = declare_branch_merge("candidate", "main").expect("merge declares");
    let context = branch_merge(&workspace, &declaration).expect("merge authority admits");
    let outcome = declaration.using(context).run(&mut workspace);
    let deferred = outcome
        .settlement_deferred()
        .expect("performed post-installation merge returns a deferred outcome");
    let settlement = deferred.settlement().clone();

    let mut foreign = primary_graph_merge_runtime();
    assert!(matches!(
        foreign.repair_deferred_branch_merge_settlement(deferred),
        Err(crate::runtime::WorthQuerySettlementRepairError::Settlement(
            worth_relational::facade::publication::DeferredPublicationSettlementError::ForeignRuntime {
                ..
            }
        ))
    ));

    let repaired = workspace
        .repair_deferred_branch_merge_settlement(deferred)
        .expect("post-installation public owner repairs settlement");
    let repeated = workspace
        .repair_deferred_branch_merge_settlement(deferred)
        .expect("post-installation repair is idempotent");
    assert_eq!(repaired.commit_id, settlement.commit().commit_id);
    assert_eq!(repeated, repaired);

    workspace
        .into_runtime()
        .write(insert_command(
            "Task",
            [(
                "identity.id",
                test_string_aspect_value("after-public-repair"),
            )],
        ))
        .expect("a subsequent public operation proceeds after repair");
}

fn primary_graph_merge_runtime() -> crate::runtime::WorthQueryRuntime {
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
        .build()
        .expect("post-installation bridge runtime publishes")
}

fn prepare_post_installation_merge_fault(runtime: &mut crate::runtime::WorthQueryRuntime) {
    runtime
        .write(insert_command(
            "Task",
            [("identity.id", test_string_aspect_value("merge-baseline"))],
        ))
        .expect("public write seeds the installed main branch");
    let graph =
        worth_query_execution::facade::integration::retain_primary_graph_integration_handle(
            &runtime.execution_runtime,
        )
        .expect("installed runtime retains its primary graph");
    graph
        .execute_mutation_with_index_refresh(|relational| {
            let main = worth_relational::facade::history::BranchId("main".to_owned());
            let (_, basis) = relational
                .observe_fork_source(&main)
                .map_err(|error| format!("fork source unavailable: {error:?}"))?;
            relational
                .fork_branch(
                    worth_relational::facade::history::BranchId("candidate".to_owned()),
                    basis,
                )
                .map_err(|error| format!("candidate fork denied: {error:?}"))?;
            Ok::<_, String>(())
        })
        .expect("fixture fork does not disturb indexes")
        .expect("fixture fork admits");
    runtime
        .write(insert_command(
            "Task",
            [(
                "identity.id",
                test_string_aspect_value("merge-source-change"),
            )],
        ))
        .expect("public write advances only the source branch");
    graph
        .execute_mutation_with_index_refresh(|relational| {
            relational.fail_next_durable_append_for_test();
            Ok::<_, String>(())
        })
        .expect("arming the fault does not disturb indexes")
        .expect("durable fault arms");
}

#[test]
fn mixed_host_schema_basis_is_rejected_before_primary_graph_publication() {
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
        .build();
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
        .build();
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
