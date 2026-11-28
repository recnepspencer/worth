use std::sync::Arc;

use worth_execution::time::ControlledTime;
use worth_foundational::time::ClockSourceIdentity;

use crate::{Rng, Spec, Trace};

/// Owns one fresh simulation's scheduling stream, clock and trace.
pub struct DstRunner {
    rng: Rng,
    clock: Arc<ControlledTime>,
    trace: Trace,
}

impl DstRunner {
    /// Runs or reproduces the scenario. The callback owns enabled operation
    /// order, domain fault choices, and the trace row schema. The first
    /// callback error stops the run and returns to the court.
    pub fn run<E>(
        spec: &Spec,
        mut step: impl FnMut(usize, &mut Self) -> Result<(), E>,
    ) -> Result<Trace, E> {
        let mut runner = Self {
            rng: spec.seed.rng(),
            clock: Arc::new(ControlledTime::with_source_identity(
                spec.start_unix_secs,
                ClockSourceIdentity::new(spec.seed.value()),
            )),
            trace: Trace::default(),
        };
        for index in 0..spec.steps {
            for fault in spec.clock_faults(index) {
                fault.apply(&runner.clock);
            }
            step(index, &mut runner)?;
        }
        Ok(runner.trace)
    }

    /// Picks from the caller's enabled operations in their canonical order.
    /// Use [`Rng::pick`] on this same stream for seeded domain fault choices.
    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Shares the run's clock with the Execution time sampler.
    pub fn clock(&self) -> Arc<ControlledTime> {
        Arc::clone(&self.clock)
    }

    pub fn trace(&mut self) -> &mut Trace {
        &mut self.trace
    }
}

#[cfg(test)]
mod tests;
