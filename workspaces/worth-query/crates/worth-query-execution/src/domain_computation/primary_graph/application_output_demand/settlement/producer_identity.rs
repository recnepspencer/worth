//! Settlement producer text keeps either its ordinary custody or recovered payer.
use crate::domain_computation::primary_graph::{
    application_checkpoint::CheckpointMetadataFunding,
    application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
};
use std::sync::Arc;
pub enum SettlementProducerIdentity {
    Current(String),
    HistoricalSelection(worth_relational::facade::mvcc::RelationalRequiredOutputPublication),
    Recovered {
        text: Arc<String>,
        _funding: Option<Arc<CheckpointMetadataFunding>>,
    },
}
impl From<String> for SettlementProducerIdentity {
    fn from(text: String) -> Self {
        Self::Current(text)
    }
}
impl SettlementProducerIdentity {
    pub(super) fn from_checkpoint(identity: &WorthQueryAcceptedOutputCheckpointIdentity) -> Self {
        Self::Recovered {
            text: Arc::clone(&identity.producer),
            _funding: identity.metadata.clone(),
        }
    }
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Current(text) => text,
            Self::HistoricalSelection(publication) => publication.producer(),
            Self::Recovered { text, .. } => text,
        }
    }
}

mod court_doors_for_test;
