//! Owning phase dispatch around the one Bridge delivery algorithm.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    OutputPublicationView, PreparedReadyBacking,
};
pub(super) enum Publication {
    Fresh(crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt),
    Recovered(crate::domain_computation::primary_graph::WorthQueryRecoveredProducerPublication),
}
impl From<crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt>
    for Publication
{
    fn from(
        value: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Self {
        Self::Fresh(value)
    }
}
impl Publication {
    pub(super) fn view(&self) -> OutputPublicationView<'_> {
        match self {
            Self::Fresh(receipt) => receipt.into(),
            Self::Recovered(publication) => publication.view(),
        }
    }
    pub(super) fn published(
        self,
        delivery: Delivery,
        ready_backing: PreparedReadyBacking,
    ) -> Checkpoint {
        match self {
            Self::Fresh(receipt) => Checkpoint::Published {
                receipt,
                delivery,
                ready_backing,
            },
            Self::Recovered(publication) => Checkpoint::RecoveredPublished {
                publication,
                delivery,
                ready_backing,
            },
        }
    }
    pub(super) fn delivered(
        self,
        delivery: Option<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        ready_backing: PreparedReadyBacking,
    ) -> Checkpoint {
        match self {
            Self::Fresh(receipt) => Checkpoint::Delivered {
                receipt,
                delivery,
                ready_backing,
            },
            Self::Recovered(publication) => Checkpoint::RecoveredDelivered {
                publication,
                delivery,
                ready_backing,
            },
        }
    }
}
