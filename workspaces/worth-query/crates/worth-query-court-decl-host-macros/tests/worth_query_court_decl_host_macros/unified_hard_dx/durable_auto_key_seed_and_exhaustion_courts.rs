//! Courts covering durable canonical auto-key seed invariance across
//! Directory Home reopen and typed probe-exhaustion denial without
//! double-applying recovered commits.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, WorthQuerySessionRunDenial,
};

use super::actions::{
    take_captured_auto_key_seed, ConstantAutoKey, ExhaustedKeyTaskState, ProbeOccupiedTaskState,
    SetTaskState,
};
use super::courts::{open_daymark, temp_home_path};
use super::program::owner;
use super::queries::ReadTaskDetails;
use super::serial_policy_and_custody_boundary_courts::open_memory_home_with_policy;

#[test]
fn auto_key_seed_is_identical_across_directory_reopen() {
    let path = temp_home_path("sol-r2-durable-seed");
    let clock = WorthQueryAdmissionClock::system();

    // 1. Create Directory Home, commit once, then close cleanly.
    {
        let home = open_daymark(&path, &clock);
        {
            let me = owner(&home, &clock);
            let _ = home
                .session(&me)
                .run(SetTaskState {
                    task: 1,
                    state: "REVIEW".to_owned(),
                })
                .unwrap();
        }
        home.close_clean().unwrap();
    }

    // 2. Reopen #1: capture the exact 32-byte durable seed on `"main"` without
    //    mutating `"main"` (missing task scope 9999 refuses after deriving probe 0).
    let seed_reopen_1 = {
        let home = open_daymark(&path, &clock);
        let seed = {
            let me = owner(&home, &clock);
            let _ = take_captured_auto_key_seed();
            let err = home
                .session(&me)
                .run(ProbeOccupiedTaskState {
                    task: 9999,
                    state: "NOOP".to_owned(),
                })
                .expect_err("missing scope refuses without mutating main");
            assert!(matches!(err, WorthQuerySessionRunDenial::Request(_)));
            take_captured_auto_key_seed().expect("probe 0 captured durable seed on reopen 1")
        };
        home.close_clean().unwrap();
        seed
    };

    // 3. Reopen #2 on the EXACT SAME durable head: seed MUST be bit-for-bit identical!
    let seed_reopen_2 = {
        let home = open_daymark(&path, &clock);
        let seed = {
            let me = owner(&home, &clock);
            let _ = take_captured_auto_key_seed();
            let err = home
                .session(&me)
                .run(ProbeOccupiedTaskState {
                    task: 9999,
                    state: "NOOP".to_owned(),
                })
                .expect_err("missing scope refuses without mutating main");
            assert!(matches!(err, WorthQuerySessionRunDenial::Request(_)));
            take_captured_auto_key_seed().expect("probe 0 captured durable seed on reopen 2")
        };
        home.close_clean().unwrap();
        seed
    };

    assert_eq!(
        seed_reopen_1, seed_reopen_2,
        "reopening the exact same durable Directory Home head must yield the exact same auto-key seed"
    );
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn auto_key_probe_exhaustion_returns_typed_denial() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home_with_policy(&clock, None);
    let me = owner(&home, &clock);
    let session = home.session(&me);

    // Occupy the single constant key returned by every probe slot of ConstantAutoKey.
    let _ = session
        .run_with_key(
            ExhaustedKeyTaskState {
                task: 1,
                state: "OCCUPIED".to_owned(),
            },
            &ConstantAutoKey(1),
        )
        .expect("explicit key occupies constant probe slot");

    // Exhausting all probe slots with a DIFFERENT intent returns AutoIdempotencyKeysExhausted.
    let diff_err = session
        .run(ExhaustedKeyTaskState {
            task: 1,
            state: "SHOULD-NOT-APPLY".to_owned(),
        })
        .expect_err("exhausted probe slots must refuse with AutoIdempotencyKeysExhausted");
    assert!(
        matches!(
            diff_err,
            WorthQuerySessionRunDenial::AutoIdempotencyKeysExhausted
        ),
        "expected AutoIdempotencyKeysExhausted (different intent), got {diff_err:?}"
    );

    // Exhausting all probe slots with an IDENTICAL intent also returns AutoIdempotencyKeysExhausted.
    let same_err = session
        .run(ExhaustedKeyTaskState {
            task: 1,
            state: "OCCUPIED".to_owned(),
        })
        .expect_err(
            "exhausted probe slots (identical intent) must refuse with AutoIdempotencyKeysExhausted",
        );
    assert!(
        matches!(
            same_err,
            WorthQuerySessionRunDenial::AutoIdempotencyKeysExhausted
        ),
        "expected AutoIdempotencyKeysExhausted (identical intent), got {same_err:?}"
    );
    assert_eq!(
        session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
        "OCCUPIED"
    );
}
