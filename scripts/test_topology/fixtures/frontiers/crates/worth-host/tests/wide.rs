#[path = "support/mod.rs"]
mod support;
#[test]
fn distinct_frontiers_remain_distinct() {
    assert_eq!(worth_host::host_value() + worth_extra::extra_value(), 5);
    assert_eq!(support::shared_value(), 7);
    assert_eq!(include_str!("fixtures/message.txt"), "fixture\n");
}
