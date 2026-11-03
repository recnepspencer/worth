//! Repeated fragment scopes inherit immutable cancellation storage cheaply.
use super::{KernelContext, RunLimits};
use crate::CancellationSource;
use std::cell::Cell;

thread_local! {
    static MATERIALIZATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}
pub(super) fn materialized() {
    MATERIALIZATIONS.update(|count| count.map(|count| count + 1));
}
struct Observation;
impl Observation {
    fn start() -> Self {
        MATERIALIZATIONS.with(|count| assert_eq!(count.replace(Some(0)), None));
        Self
    }
    fn count(&self) -> usize {
        MATERIALIZATIONS.with(|count| count.get().unwrap())
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        MATERIALIZATIONS.set(None);
    }
}

#[test]
fn repeated_fragment_scopes_do_not_reconstruct_cancellation_storage() {
    let cancellation = CancellationSource::new();
    let mut limits = RunLimits::for_run(None, false);
    limits.tokens.push(cancellation.token());
    let context = KernelContext::new(limits);
    let _parent = context.enter();
    for (fragments, cumulative_work) in [(8, 8), (64, 72), (512, 584)] {
        let observation = Observation::start();
        let payloads = crate::backend::scope::cost::Observation::start();
        for _ in 0..fragments {
            let mut inherited = RunLimits::for_run(None, false);
            inherited.tokens.push(cancellation.token());
            assert_eq!(inherited.tokens.len(), 2);
            assert!(!inherited.tokens.iter().any(|token| token.is_cancelled()));
            crate::backend::scope::run_scope_native_within::<(), ()>(
                None,
                true,
                None,
                None,
                |_| Ok(()),
            )
            .result
            .unwrap();
            crate::backend::scope::run_scope_unit_within(None, None, None, |context| {
                context.checkpoint(1)?;
                Ok(())
            })
            .result
            .unwrap();
        }
        let materializations = observation.count();
        eprintln!("fragments={fragments} cancellation materializations={materializations}");
        assert_eq!(materializations, 0, "fragments={fragments}");
        assert_eq!(context.cost().0, cumulative_work);
        assert_eq!(payloads.count(), 0, "fragments={fragments}");
    }
}

#[test]
#[cfg(all(target_pointer_width = "64", target_os = "windows"))]
fn framework_memory_quotes_keep_unfixed_literals() {
    assert_eq!(size_of::<RunLimits>(), 112);
    assert_eq!(size_of::<super::KernelState>(), 200);
    let cancellation = CancellationSource::new();
    let mut limits = RunLimits::for_run(None, false);
    for expected in [384, 400, 416, 432] {
        assert_eq!(limits.framework_context_bytes(1, 1), Some(expected));
        limits.tokens.push(cancellation.token());
    }
}
