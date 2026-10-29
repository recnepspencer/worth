pub(crate) use worth_query_worlds::facade::certification::application_graph::adoption::migration::unowned_binding::*;

#[test]
fn target_program_must_own_the_exact_migration_binding() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let denial =
        host.runtime()
            .with_application_advancement(&scope, |phase| {
                host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .advanced().mutate(UnownedMigrationIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 15,
                    },
                })
                .without_source()
                .idempotency(&0x9175_2100)
                .prepare_program_migration(
                    &phase,
                    &target,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("operation identity cannot substitute for exact binding ownership")
            })
            .expect("the installed host admits migration preparation");
    assert!(matches!(
        denial,
        WorthQueryApplicationProgramMigrationPreparationDenial::Migration(
            worth_query_host::facade::primary_graph::WorthQueryProgramMigrationPreparationDenial::TargetBindingUnowned
        )
    ));
}
