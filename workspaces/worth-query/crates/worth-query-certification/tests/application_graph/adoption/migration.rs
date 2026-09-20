//! A typed mutation candidate repairs incompatible state inside adoption's
//! single Relational publication rather than becoming a separate operation.

#[path = "migration/unowned_binding.rs"]
mod unowned_binding;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramMigrationPreparationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryBranchAdoptionRecoveryOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

use crate::bounded_dimension_model::dimension_entry::{
    candidate_count, reset_candidate_count, SetPartDimensionIntent,
    MIGRATION_CANDIDATE_PROBE_DIMENSION, PART_IDENTITY,
};
use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::DimensionProgramP1;
use crate::bounded_dimension_model::readback::read_dimension;
use crate::bounded_dimension_model::schema::SetPartDimensionInput;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

const P0_ONLY_DIMENSION: u64 = 3;
const P1_VALID_DIMENSION: u64 = 15;

#[test]
fn typed_migration_repairs_state_inside_the_adoption_publication() {
    let host = publish_on_first_program();
    let initial = host.current_world();
    assert_eq!(
        settle(set_dimension(
            &host,
            initial,
            P0_ONLY_DIMENSION,
            0x9175_2101,
        )),
        DimensionVerdict::Performed(P0_ONLY_DIMENSION),
    );
    let branch = host.current_world();
    let sibling = host
        .runtime()
        .branches()
        .fork(branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the source-program sibling must publish");
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let migration = match host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(SetPartDimensionIntent {
            input: SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: P1_VALID_DIMENSION,
            },
        })
        .without_source()
        .idempotency(&0x9175_2102)
        .prepare_program_migration(&target)
        .expect("the target-owned migration must prepare")
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
        .adopt(&target)
        .requirements(&requirements)
        .migration(migration)
        .prepare(64)
        .expect("migration and target validation must prepare atomically");
    assert!(prepared.migration().is_some());
    let performed = match prepared.publish() {
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
        read_dimension(host.runtime(), host.current_world()),
        P1_VALID_DIMENSION
    );
    assert_eq!(
        read_dimension(host.runtime(), sibling),
        P0_ONLY_DIMENSION,
        "migration and target activation must remain local to the selected branch"
    );
}

#[test]
fn migration_candidate_cannot_cross_the_source_product_head() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let migration = match host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(SetPartDimensionIntent {
            input: SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 16,
            },
        })
        .without_source()
        .idempotency(&0x9175_2111)
        .prepare_program_migration(&target)
        .expect("migration prepares on the original source")
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
        settle(set_dimension(&host, branch, 8, 0x9175_2112)),
        DimensionVerdict::Performed(8),
    );

    let denial = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .adopt(&target)
        .requirements(&requirements)
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
    assert_eq!(read_dimension(host.runtime(), host.current_world()), 8);
}

#[test]
fn unpublished_migration_recovery_never_reruns_candidate_authoring() {
    let host = publish_on_first_program();
    let initial = host.current_world();
    assert_eq!(
        settle(set_dimension(
            &host,
            initial,
            P0_ONLY_DIMENSION,
            0x9175_2121,
        )),
        DimensionVerdict::Performed(P0_ONLY_DIMENSION),
    );
    let branch = host.current_world();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    reset_candidate_count(MIGRATION_CANDIDATE_PROBE_DIMENSION);
    let migration = match host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(SetPartDimensionIntent {
            input: SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: MIGRATION_CANDIDATE_PROBE_DIMENSION,
            },
        })
        .without_source()
        .idempotency(&0x9175_2122)
        .prepare_program_migration(&target)
        .expect("migration prepares")
    {
        WorthQueryApplicationProgramMigrationPreparationOutcome::Prepared(prepared) => prepared,
        _ => panic!("migration must complete"),
    };
    assert_eq!(candidate_count(MIGRATION_CANDIDATE_PROBE_DIMENSION), 1);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .migration(migration)
        .prepare(64)
        .expect("migration adoption prepares");

    host.runtime().fail_next_durable_append_for_test();
    let unpublished = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => unpublished,
        _ => panic!("the injected durable append failure must retain custody"),
    };
    assert!(unpublished.migration().is_some());
    let recovery = unpublished.into_recovery();
    assert!(recovery.migration().is_some());
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the exact migration adoption must recover"));
    let performed = match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } => {
            cleanup.expect("recovery cleanup drains");
            adoption
        }
        _ => panic!("settled migration adoption must perform"),
    };
    assert!(performed.migration().is_some());
    assert_eq!(performed.relational_owner_contacts(), 0);
    assert_eq!(candidate_count(MIGRATION_CANDIDATE_PROBE_DIMENSION), 1);
    assert_eq!(
        read_dimension(host.runtime(), host.current_world()),
        MIGRATION_CANDIDATE_PROBE_DIMENSION,
    );
}
