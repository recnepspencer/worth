//! Width is witnessed by computing members, including a perturbed real wave.
use super::*;
use std::sync::Condvar;
const CHARGES: [u64; 2] = [3, 7];
const HANG_GUARD: Duration = Duration::from_secs(60);
#[test]
fn certified_wave_really_overlaps_computing_members() {
    let _owned=crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    super::super::super::super::request_execution::place_managed_computations_on_this_thread_for_test(super::super::super::super::request_execution::WorthQueryExecutionPlacementForTest::Certified{workers:NonZeroUsize::new(2).unwrap(),seed:0});
    let state = Arc::new((Mutex::new((0usize, 0usize, 0usize, false)), Condvar::new()));
    let members = CHARGES
        .into_iter()
        .map(|units| {
            let state = state.clone();
            PreparedMember {
                requests: vec![PreparedCompute::new(move |member| {
                    let (mut held, wake) = (state.0.lock().unwrap(), &state.1);
                    held.0 += 1;
                    held.1 += 1;
                    held.2 = held.2.max(held.1);
                    wake.notify_all();
                    let (mut held, guard) = wake
                        .wait_timeout_while(held, HANG_GUARD, |s| s.0 < 2)
                        .unwrap();
                    held.3 |= guard.timed_out() && held.0 < 2;
                    drop(held);
                    let (_, mut charge) = member.into_parts();
                    charge.as_mut().unwrap()(units).unwrap();
                    state.0.lock().unwrap().1 -= 1;
                    units
                })],
            }
        })
        .collect();
    let (settled, report) = run(Some(2), members);
    let state = state.0.lock().unwrap();
    assert_eq!(
        state.2, 2,
        "leased certified wave has two computing members in flight"
    );
    assert!(!state.3, "computing-member rendezvous hang guard expired");
    assert_eq!(settled.len(), 2);

    assert_eq!(report.charged_span(), *CHARGES.iter().max().unwrap());
}
