//! The same actual provider charges and stops without an invariant registration.
use super::*;

#[test]
fn public_only_source_preserves_original_work_exhaustion_and_inherited_cancellation() {
    let _serial = crate::leased_preparation::ordinary_context::serial();
    for mode in [Mode::Serial, Mode::Leased, Mode::Inherited] {
        let scenario = Scenario {
            public_only: true,
            ..Default::default()
        };
        let base = sample(mode, scenario, EXTRA - 1);
        assert_eq!(base.terminal, Terminal::Committed);
        assert_eq!(base.reached, 1);
        let charged = sample(
            mode,
            Scenario {
                custom: EXTRA,
                ..scenario
            },
            1_000_000,
        );
        assert_eq!(charged.terminal, Terminal::Committed);
        assert_eq!(charged.work, base.work + EXTRA);
        let stopped = sample(
            mode,
            Scenario {
                custom: EXTRA,
                ..scenario
            },
            EXTRA - 1,
        );
        assert_eq!(
            stopped.terminal,
            Terminal::Request(RelationalExecutionDenialCause::WorkExhausted)
        );
        assert_eq!(stopped.reached, 1);
        assert!(stopped.work <= EXTRA - 1);
        assert!(stopped.work > base.work);
    }
    let cancelled = sample(
        Mode::Inherited,
        Scenario {
            public_only: true,
            cancel_parent: true,
            ..Default::default()
        },
        1_000_000,
    );
    assert_eq!(
        cancelled.terminal,
        Terminal::Request(RelationalExecutionDenialCause::Cancelled)
    );
    assert_eq!(cancelled.reached, 1);
    assert!(cancelled.work > 0);
}

#[test]
fn public_registration_cannot_bypass_the_same_programs_installed_rule_allowance() {
    let _serial = crate::leased_preparation::ordinary_context::serial();
    let allowed = sample(
        Mode::Serial,
        Scenario {
            public_also: true,
            ..Default::default()
        },
        1_000_000,
    );
    assert_eq!(allowed.terminal, Terminal::Committed);
    assert_eq!(allowed.reached, 1);
    let limited = sample(
        Mode::Serial,
        Scenario {
            public_also: true,
            rule_limit: true,
            ..Default::default()
        },
        1_000_000,
    );
    assert_eq!(limited.terminal, Terminal::RuleLimit);
    assert_eq!(limited.reached, 0);
}
