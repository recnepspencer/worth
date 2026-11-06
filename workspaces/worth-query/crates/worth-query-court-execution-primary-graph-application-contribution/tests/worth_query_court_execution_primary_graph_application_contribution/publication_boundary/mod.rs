//! The publication boundary's document history constraint stops, through the moved helpers.
use worth_query_execution::publication_boundary::program_publication_access;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::product_operation::document_history::constraint::installed_entry_tests::InstalledEntryTestsWorthQueryDocumentHistoryOperationConstraintForTest;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::product_operation::document_history::constraint::ownership_tests::OwnershipTestsWorthQueryDocumentHistoryOperationConstraintForTest;

#[test]
fn foreign_installed_seal_is_stale_at_history_constraint_before_work() {
    worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperationConstraint::assert_foreign_installed_seal_is_stale_before_history_work(
        &program_publication_access(),
    );
}

#[test]
fn entry_custody_foreign_history_model_precedes_authentication_lapse() {
    worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperationConstraint::assert_foreign_history_model_precedes_authentication_lapse(
        &program_publication_access(),
    );
}
