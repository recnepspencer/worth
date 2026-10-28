//! Missing arrivals fail the rendezvous and release every peer.
use super::*;
fn control() -> Control {
    let world = super::super::seeded_world::world(super::super::Family::Empty, super::super::SEED);
    let mut control = Control::new(world, false, true, 0, true);
    control.backstop = Duration::from_millis(250);
    control.state.lock().unwrap().armed = true;
    control
}
fn message(panic: Box<dyn std::any::Any + Send>) -> String {
    *panic
        .downcast::<String>()
        .expect("named rendezvous timeout")
}
#[test]
fn missing_wave_member_is_named_and_all_peers_are_released() {
    let control = control();
    control.state.lock().unwrap().member_rendezvous = vec![vec![10, 20, 30, 40]];
    std::thread::scope(|scope| {
        let control = &control;
        let peers = [10, 20, 30]
            .map(|key| scope.spawn(move || std::panic::catch_unwind(|| control.dispatched(key))));
        for peer in peers {
            let stop = message(peer.join().unwrap().expect_err("member 40 never arrives"));
            assert!(
                stop.contains("wave member rendezvous backstop expired") && stop.contains("40"),
                "{stop}"
            );
        }
    });
}
#[test]
fn missing_kernel_member_is_named_and_releases_the_wave_waiter() {
    // Prove the kernel's own expiry without a competing timeout. With both
    // wait kinds live, either timed-out thread may acquire the mutex first;
    // scheduler wake order must not choose the test's expected answer.
    {
        let control = control();
        control.state.lock().unwrap().witnesses = vec![10, 20];
        let stop = message(
            std::panic::catch_unwind(|| control.enter_compute(10, 0, 1))
                .expect_err("kernel member 20 never arrives"),
        );
        assert_eq!(
            stop,
            "kernel overlap rendezvous backstop expired: missing members [20]"
        );
    }
    let control = control();
    {
        let mut state = control.state.lock().unwrap();
        state.witnesses = vec![10, 20];
        state.member_rendezvous = vec![vec![30, 40]];
    }
    std::thread::scope(|scope| {
        let kernel = scope.spawn(|| std::panic::catch_unwind(|| control.enter_compute(10, 0, 1)));
        let wave = scope.spawn(|| std::panic::catch_unwind(|| control.dispatched(30)));
        let kernel = message(
            kernel
                .join()
                .unwrap()
                .expect_err("missing kernel or wave member stops both wait kinds"),
        );
        let wave = message(
            wave.join()
                .unwrap()
                .expect_err("missing kernel or wave member stops both wait kinds"),
        );
        assert!(
            matches!(
                kernel.as_str(),
                "kernel overlap rendezvous backstop expired: missing members [20]"
                    | "wave member rendezvous backstop expired: missing members [40]"
            ),
            "{kernel}"
        );
        assert_eq!(kernel, wave, "all waiters propagate the same first failure");
    });
}
