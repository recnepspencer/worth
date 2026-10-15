use super::*;

#[test]
fn mutation_audit_rows_map_to_implemented_coverage_inventory_entrypoints() {
    let audit = worth_query_intent_admission_mutation_audit();
    let inventory = worth_query_intent_admission_coverage_inventory();
    let implemented = inventory
        .rows()
        .iter()
        .filter(|row| row.status() == WorthQueryIntentAdmissionCoverageStatus::Implemented)
        .collect::<Vec<_>>();
    let mut proof_cases = std::collections::HashSet::new();

    for row in audit.rows() {
        assert_eq!(
            row.family(),
            WorthQueryIntentAdmissionFamily::AuthoritativeMutationIntent
        );
        assert!(implemented.iter().any(|coverage| {
            coverage.entrypoint() == row.entrypoint() && coverage.family() == row.family()
        }));
        assert!(
            proof_cases.insert(row.proof_case()),
            "duplicate mutation audit proof case for {}",
            row.public_surface()
        );
        assert!(!row.delegation_evidence().is_empty());
    }
}
