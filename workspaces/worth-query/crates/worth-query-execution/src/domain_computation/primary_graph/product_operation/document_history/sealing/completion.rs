//! A completion retains its actual live or recorded source class.
use super::super::{
    source_evidence::WorthQueryHistorySourceEvidence, WorthQueryCompletedRecordingOutputs,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryCompletedRequiredOutputs,
    WorthQueryRecoveredApplicationCommitReceipt,
};

pub(in super::super) enum RecordingCompletion<'a> {
    Source(&'a WorthQueryApplicationCommitReceipt),
    RecordedSource(&'a WorthQueryRecoveredApplicationCommitReceipt),
    RequiredOutputs {
        source: &'a WorthQueryApplicationCommitReceipt,
        completed: &'a WorthQueryCompletedRequiredOutputs,
    },
    Forest {
        source: WorthQueryHistorySourceEvidence<'a>,
        required: Option<&'a WorthQueryCompletedRequiredOutputs>,
        completed: &'a WorthQueryCompletedRecordingOutputs,
    },
}
impl RecordingCompletion<'_> {
    pub(super) fn source(&self) -> WorthQueryHistorySourceEvidence<'_> {
        match self {
            Self::Source(source) | Self::RequiredOutputs { source, .. } => {
                WorthQueryHistorySourceEvidence::Live(source)
            }
            Self::Forest { source, .. } => *source,
            Self::RecordedSource(source) => WorthQueryHistorySourceEvidence::Recorded(source),
        }
    }
}
