//! Negative court proving a mismatched due-field binding fails closed before commit (Blocker 3).

use crate::host::temporal_catchup::pump::*;

use worth_query_host::facade::{
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpStatus},
};

use super::{
    adapters::{AuthControl, ReminderFireLog},
    program::{mutate_reminder, open_memory_home_with_mismatched_due_field, read_reminders},
    schema::MutateReminderInput,
    with_leased_request,
};

#[test]
fn desktop_temporal_mismatched_due_field_binding_fails_closed_before_commit() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    with_leased_request(128 * 1024 * 1024, None, |execution| {
        let home =
            open_memory_home_with_mismatched_due_field(fire_log.clone(), auth.clone(), execution);
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i1".to_owned(),
                revision: 1,
                due: 10,
                interval: 5,
                max_fires: 2,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        let report = catch_up(
            &home,
            WorthQueryClockCoordinate::from_nanoseconds(15),
            execution,
            WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
        )
        .unwrap();
        assert_eq!(
            report.status(),
            &WorthQueryTemporalCatchUpStatus::OperationRefused,
            "mismatched due field must fail closed before commit: {report:?}"
        );
        assert_eq!(report.fired(), 0);
        assert!(report.remaining());
        assert!(fire_log.snapshot().is_empty());
        let rows = read_reminders(&home);
        assert_eq!(rows["i1"].revision, 1);
        assert_eq!(rows["i1"].due, 10);
        assert_eq!(rows["i1"].interval, 5);
        assert_eq!(rows["i1"].fired_count, 0);
    });
}
