//! A typed mutation candidate repairs incompatible state inside adoption's
//! single Relational publication rather than becoming a separate operation.

#[path = "migration/unowned_binding.rs"]
mod unowned_binding;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramMigrationPreparationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

use crate::document_retention_model::host::publish_on_first_program;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::programs::RetentionProgramP1;
use crate::document_retention_model::readback::read_retention;
use crate::document_retention_model::retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY};
use crate::document_retention_model::schema::SetRetentionInput;
use crate::document_retention_model::settled_verdict::{settle, RetentionVerdict};

const P0_ONLY_RETENTION: u64 = 3;
const P1_VALID_RETENTION: u64 = 15;

#[test]
fn typed_migration_repairs_state_inside_the_adoption_publication() {
    let host = publish_on_first_program();
    let initial = host.current_world();
    assert_eq!(
        settle(set_retention(
            &host,
            initial,
            P0_ONLY_RETENTION,
            0x9175_2101,
        )),
        RetentionVerdict::Performed(P0_ONLY_RETENTION),
    );
    let branch = host.current_world();
    let sibling = host
        .with_host_advancement(|host_phase| {
            host.runtime()
                .branches()
                .fork(branch)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("the source-program sibling must publish")
        })
        .expect("installed host call opens");
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let migration = match host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .advanced()
                .mutate(SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: P1_VALID_RETENTION,
                    },
                })
                .without_source()
                .idempotency(&0x9175_2102)
                .prepare_program_migration(
                    &phase,
                    &target,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("the target-owned migration must prepare")
        })
        .expect("the installed host admits migration preparation")
    {
        WorthQueryApplicationProgramMigrationPreparationOutcome::Prepared(prepared) => prepared,
        WorthQueryApplicationProgramMigrationPreparationOutcome::DomainDenied(denial) => {
            panic!("migration was domain denied: {denial:?}")
        }
        WorthQueryApplicationProgramMigrationPreparationOutcome::Cancelled => {
            panic!("migration was cancelled")
        }
        WorthQueryApplicationProgramMigrationPreparationOutcome::DeadlineExceeded => {
            panic!("migration exceeded its deadline")
        }
    };
    assert_eq!(migration.target(), &target);
    assert_eq!(migration.description().effect_count(), 1);

    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison must succeed");
    let prepared = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .migration(migration)
        .prepare(64)
        .expect("migration and target validation must prepare atomically");
    assert!(prepared.migration().is_some());
    let performed = match prepared.publish(host.runtime(), &principal, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, .. } => {
            panic!("authorized adoption refused: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, .. } => {
            panic!("execution admission refused: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
            panic!("the declared host policy admits adoption: {cause:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => performed,
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
            panic!("migration adoption unexpectedly had no effect: {no_effect:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            panic!("migration adoption lost publication: {unpublished:?}")
        }
    };
    assert_eq!(performed.target(), &target);
    assert!(performed.migration().is_some());
    assert_eq!(
        read_retention(host.runtime(), host.current_world()),
        P1_VALID_RETENTION
    );
    assert_eq!(
        read_retention(host.runtime(), sibling),
        P0_ONLY_RETENTION,
        "migration and target activation must remain local to the selected branch"
    );
}

#[test]
fn migration_candidate_cannot_cross_the_source_product_head() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let migration = match host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .advanced()
                .mutate(SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 16,
                    },
                })
                .without_source()
                .idempotency(&0x9175_2111)
                .prepare_program_migration(
                    &phase,
                    &target,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("migration prepares on the original source")
        })
        .expect("the installed host admits migration preparation")
    {
        WorthQueryApplicationProgramMigrationPreparationOutcome::Prepared(prepared) => prepared,
        _ => panic!("migration must complete before the source moves"),
    };
    let requirements = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .compare(&target)
        .expect("comparison succeeds");
    assert_eq!(
        settle(set_retention(&host, branch, 8, 0x9175_2112)),
        RetentionVerdict::Performed(8),
    );

    let denial = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .migration(migration)
        .prepare(64)
        .err()
        .expect("the moved source must reject the sealed migration");
    assert!(matches!(
        denial,
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::MigrationSourceChanged
        )
    ));
    assert_eq!(read_retention(host.runtime(), host.current_world()), 8);
}
