//! Brand checks compare copied identities; they never allocate.

use super::*;

const PROBE: &str = "WORTH_QUERY_RUN_ALLOCATION_PROBE";

#[test]
fn brand_checks_do_not_allocate() {
    let output = crate::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "--test-threads=1", PROBE_TEST])
            .env(PROBE, "1"),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

const PROBE_TEST: &str = "domain_computation::primary_graph::tests::application_attempt::erased_core_forgery::allocation::isolated_brand_check_allocation_probe";

#[test]
fn isolated_brand_check_allocation_probe() {
    if std::env::var_os(PROBE).is_none() {
        return;
    }
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (principal, account) = party(&world, &request);
    let admission = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    let runtime = world.application.runtime.authority_identity();
    let binding = world.application.installed_schema.binding_identity();
    let descriptor = world
        .application
        .installed_schema
        .installed_mutation_binding_descriptor(ProgramRequiredMutationBinding::IDENTITY);

    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    for _ in 0..64 {
        std::hint::black_box(admission.validate_installed_entry(
            runtime,
            &binding,
            &world.application.runtime.installed_operations,
        ))
        .unwrap();
        assert!(std::hint::black_box(
            admission.belongs_to(runtime, &binding)
        ));
        std::hint::black_box(
            admission.validate_installed_operation(&world.application.runtime.installed_operations),
        )
        .unwrap();
        std::hint::black_box(admission.validate_installed_mutation_binding(descriptor)).unwrap();
        std::hint::black_box(admission.capability_input());
    }
    let change = region.change();
    assert_eq!(
        (change.allocations, change.reallocations),
        (0, 0),
        "{change:?}"
    );
}
