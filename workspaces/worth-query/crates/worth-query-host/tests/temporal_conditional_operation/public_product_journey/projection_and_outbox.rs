use worth_query_host::facade::{primary_graph, publication};

pub(super) fn assert_commit_publication(
    execution: &primary_graph::WorthQueryApplicationCommitReceipt,
) {
    let execution_product = execution.committed_product_publication();
    let published = publication::domain_computation::publish_application_commit(execution.clone());
    let receipt = published.receipt();
    let product = receipt.product_commit();

    assert_eq!(product.product_branch(), execution_product.product_branch());
    assert_eq!(
        product.product_incarnation(),
        execution_product.product_incarnation()
    );
    assert_eq!(
        product.product_generation(),
        execution_product.product_generation()
    );
    assert_eq!(
        product.composite_commit(),
        execution_product.composite_commit()
    );
    assert_eq!(
        product.publication_attempt(),
        execution_product.publication_attempt()
    );
    assert_eq!(
        product.relational_commit(),
        execution_product.relational_commit()
    );
    assert_eq!(
        product.relational_posture(),
        execution_product.relational_posture()
    );
    assert_eq!(product.signal_posture(), execution_product.signal_posture());
    assert_eq!(
        product.signal_publication(),
        execution_product.signal_publication()
    );
    assert_eq!(
        product.conditional_definition_generation(),
        execution_product.conditional_definition_generation()
    );
    assert_eq!(
        receipt.aftermath().external_effect().kind(),
        publication::application_aftermath::WorthQueryPublishedExternalEffectPostureKind::Completed
    );
    assert_eq!(
        receipt.inspect().kind(),
        publication::application_aftermath::WorthQueryPublishedApplicationCommitKind::Executed
    );
}
