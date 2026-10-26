//! Every wave member's entry/exit contributes to the observed concurrency peak.
use super::super::control::Control;

pub(super) fn assert_member_widths(control: &Control, waves: &[Vec<u64>], workers: usize) {
    let state = control.state.lock().unwrap();
    assert_eq!(
        state.wave_peaks.len(),
        waves.len(),
        "every computing wave is observed"
    );
    assert!(
        state.wave_active.iter().all(|active| *active == 0),
        "every wave member exits"
    );
    for (wave, peak) in waves.iter().zip(&state.wave_peaks) {
        let width = workers.min(wave.len());
        assert!(*peak <= width, "wave {wave:?}: independently measured member peak {peak} exceeds admitted width {width}");
        assert_eq!(*peak, width,
            "wave {wave:?}: independently measured member peak must equal admitted width {width}; rendezvous membership does not filter the measurement");
    }
}

#[test]
fn independently_observed_width_rejects_underdispatch_and_oversubscription() {
    // Inject actual dispatch widths 1/2/3 against a two-worker admission.
    // This intentionally uses no fixture rendezvous: the measurement must
    // remain sensitive without a selected key set defining its answer.
    for actual in [1, 2, 3] {
        let world =
            super::super::seeded_world::world(super::super::Family::Empty, super::super::SEED);
        let control = Control::new(world, false, false, 0, false);
        let waves = vec![vec![10, 20, 30, 40]];
        {
            let mut state = control.state.lock().unwrap();
            state.armed = true;
            state.member_waves = waves.clone();
            state.wave_active = vec![0];
            state.wave_peaks = vec![0];
        }
        let (entered, arrivals) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            let mut releases = Vec::new();
            for key in waves[0].iter().copied().take(actual) {
                let control = &control;
                let entered = entered.clone();
                let (release, wait) = std::sync::mpsc::channel();
                releases.push(release);
                scope.spawn(move || {
                    control.dispatched(key);
                    entered.send(key).unwrap();
                    wait.recv_timeout(std::time::Duration::from_secs(120))
                        .expect("width probe is released");
                    control.exit_member(key);
                });
            }
            for _ in 0..actual {
                arrivals
                    .recv_timeout(std::time::Duration::from_secs(120))
                    .expect("injected member arrives");
            }
            for release in releases {
                release.send(()).unwrap();
            }
        });
        for key in waves[0].iter().copied().skip(actual) {
            control.dispatched(key);
            control.exit_member(key);
        }
        assert_eq!(control.state.lock().unwrap().wave_peaks, [actual]);
        let result = std::panic::catch_unwind(|| assert_member_widths(&control, &waves, 2));
        assert_eq!(
            result.is_ok(),
            actual == 2,
            "actual dispatch width={actual}, admitted=2"
        );
        if let Err(panic) = result {
            let message = panic.downcast_ref::<String>().unwrap();
            assert!(
                message.contains("independently measured member peak"),
                "{message}"
            );
        }
    }
}
