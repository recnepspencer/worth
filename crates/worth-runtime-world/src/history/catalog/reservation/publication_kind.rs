//! Exact resident publication class, selected before reserving the entry.
use super::super::metadata::{HistoryMetadataArithmeticOverflow, HistoryMetadataCharge};
use super::*;

pub(in crate::history::catalog) enum ReservedHistoryPublication {
    NoPublication,
    Live(Arc<crate::history::CanonicalPublicationEnvelope>),
    Readmitted,
}
impl ReservedHistoryPublication {
    pub(in crate::history::catalog) fn charge(
        &self,
        base: HistoryMetadataCharge,
    ) -> Result<HistoryMetadataCharge, HistoryMetadataArithmeticOverflow> {
        match self {
            Self::NoPublication => Ok(base),
            Self::Live(publication) => base.with_publication(Some(publication)),
            Self::Readmitted => base.with_readmitted(),
        }
    }

    pub(super) fn live(&self) -> Option<&Arc<crate::history::CanonicalPublicationEnvelope>> {
        match self {
            Self::Live(publication) => Some(publication),
            Self::NoPublication | Self::Readmitted => None,
        }
    }

    pub(super) fn ordinary_evidence(
        &self,
    ) -> Result<Option<crate::history::PublicationEvidence>, CompositeHistoryCatalogDenial> {
        match self {
            Self::NoPublication => Ok(None),
            Self::Live(publication) => Ok(Some(crate::history::PublicationEvidence::Live(
                publication.clone(),
            ))),
            Self::Readmitted => Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch),
        }
    }
}
