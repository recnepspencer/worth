#[test]
fn isolated_work_keeps_the_serial_prefix() {
    assert_eq!(worth_host::host_value(), 2);
}
