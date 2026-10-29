#[test]
fn reference_model_agrees() {
    assert_eq!(crate::fast::plain::cross_category_value(), 3);
    let reference = 2;
    assert_eq!(worth_host::host_value(), reference);
}
