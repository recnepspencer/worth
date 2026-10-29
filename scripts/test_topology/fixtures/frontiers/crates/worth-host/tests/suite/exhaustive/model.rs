#[test]
fn reference_model_agrees() {
    let reference = 2;
    assert_eq!(worth_host::host_value(), reference);
}
