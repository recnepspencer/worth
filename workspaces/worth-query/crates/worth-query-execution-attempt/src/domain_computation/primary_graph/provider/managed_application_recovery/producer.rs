//! Original producer recovery uses the same retained native attempt owner.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;

pub struct ProducerRecoveryCustody {
    pub facts: std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::RecoveredProducerOutputFacts>,
    pub carrier: Option<WorthQueryRecoveredProgramOutputSource>,
    pub binding: WorthQueryApplicationIdempotencyBinding,
    pub cleanup: Option<WorthQueryProductUnpublishedRecovery>,
}
