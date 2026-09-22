use super::WorthQueryApplicationIdempotencyBinding;

fn baseline() -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
        .bind_source(Some(&[3; 32]))
        .bind_workflow_transition(&[4; 32])
        .bind_workflow_operation(&[5; 32])
}

#[test]
fn recovery_request_comparator_covers_each_preparation_axis() {
    let receipt = baseline();
    assert!(receipt.matches_recovery_request(&baseline()));
    assert!(!receipt.matches_recovery_request(
        &WorthQueryApplicationIdempotencyBinding::new([9; 32], [2; 32])
            .bind_source(Some(&[3; 32]))
            .bind_workflow_transition(&[4; 32])
            .bind_workflow_operation(&[5; 32])
    ));
    assert!(!receipt.matches_recovery_request(
        &WorthQueryApplicationIdempotencyBinding::new([1; 32], [9; 32])
            .bind_source(Some(&[3; 32]))
            .bind_workflow_transition(&[4; 32])
            .bind_workflow_operation(&[5; 32])
    ));
    assert!(!receipt.matches_recovery_request(
        &WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
            .bind_source(Some(&[9; 32]))
            .bind_workflow_transition(&[4; 32])
            .bind_workflow_operation(&[5; 32])
    ));
    assert!(!receipt.matches_recovery_request(
        &WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
            .bind_source(Some(&[3; 32]))
            .bind_workflow_transition(&[9; 32])
            .bind_workflow_operation(&[5; 32])
    ));
    assert!(!receipt.matches_recovery_request(
        &WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
            .bind_source(Some(&[3; 32]))
            .bind_workflow_transition(&[4; 32])
            .bind_workflow_operation(&[9; 32])
    ));
}
