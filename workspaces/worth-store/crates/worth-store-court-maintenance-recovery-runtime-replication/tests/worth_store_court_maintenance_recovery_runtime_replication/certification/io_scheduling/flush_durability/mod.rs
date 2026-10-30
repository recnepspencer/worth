use worth_store_certification::S6FlushDurabilityEvidenceRow;
use {
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

#[test]
fn certification_consumes_canonical_physical_completion_evidence() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize("s6-canonical-flush-evidence").unwrap();
    let acknowledgment =
        canonical_physical_mutation_acknowledgment(&world, [63; 32], b"canonical-flush-evidence");
    let row = worth_store_certification::S6FlushDurabilityEvidenceRow::from_physical_acknowledgment(
        &acknowledgment,
    );

    assert_eq!(
        row.executed_boundary().mutation_identity(),
        acknowledgment.mutation_identity()
    );
    assert_eq!(
        row.performance().mutation_identity(),
        acknowledgment.mutation_identity()
    );
    assert_eq!(
        row.performance().bytes_completed(),
        row.performance().bytes_requested()
    );
    assert_eq!(
        row.counter_strength(),
        worth_store_budgets::CounterEvidenceStrength::Exact
    );

    world.close();
}
