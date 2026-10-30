#[allow(unused_imports)]
use crate::request_differential::ordinary_context::*;

#[test]
fn serial_default_admits_three_hundred_tasks_and_reports_their_charge() {
    let (values, report) = serial(300).unwrap();
    for (index, value) in values.iter().enumerate() {
        assert_eq!(
            value.get(worth_signal::facade::Aspect::new(0)),
            index as u64 + 1
        );
    }
    assert!(report.execution.last().unwrap().charged_work() > 0);
}

#[path = "request_custody/mod.rs"]
pub(crate) mod request_custody;
