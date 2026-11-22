//! Borrowed cutover port for synchronous producer advancement.
use worth_runtime_world::facade::{PreparedWorldRelationalCutover, ProductBranchObservation};

use crate::ports::HistoryProducerCutoverPort;

#[derive(Clone, Copy)]
pub struct HistoryProducerCutover<'call> {
    port: &'call dyn HistoryProducerCutoverPort,
}

impl<'call> HistoryProducerCutover<'call> {
    pub const fn new(port: &'call dyn HistoryProducerCutoverPort) -> Self {
        Self { port }
    }

    pub fn uses_owned_model_graph(&self) -> bool {
        self.port.uses_owned_model_graph()
    }

    pub fn current(&self) -> &ProductBranchObservation {
        self.port.current()
    }

    pub fn perform(
        self,
        cutover: PreparedWorldRelationalCutover<'_, '_, '_, '_>,
        expected_target: &ProductBranchObservation,
    ) -> Result<(), ()> {
        self.port.perform(cutover, expected_target)
    }
}
