//! A mismatch describes Native's subject without retaining its publication custody.
use std::sync::Arc;
use worth_relational::facade::mvcc::{CompanionPreflightStop, RelationalRequiredOutputPublication};
mod subject_capture_for_test;
#[derive(Debug)]
pub enum HistoricalTransportDenial {
    Lookup(CompanionPreflightStop),
    PublicationCorrespondence(WorthQueryHistoryPublicationCorrespondenceMismatch),
}
/// Descriptive Query roles disagree with this selected Native publication.
#[derive(Clone)]
pub struct WorthQueryHistoryPublicationCorrespondenceMismatch {
    subject: Arc<Subject>,
}
struct Subject {
    producer: String,
    family: String,
    source: worth_relational::facade::identity::EntityId,
    coordinate: (u64, usize),
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryHistoryPublicationCorrespondenceMismatch {
    pub(super) fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        publication: &RelationalRequiredOutputPublication,
    ) -> Result<Self, CompanionPreflightStop> {
        let names = publication
            .producer()
            .len()
            .checked_add(publication.meaning().family.len())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let bytes = names
            .checked_add(std::mem::size_of::<Subject>() + 2 * std::mem::size_of::<usize>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            execution,
            execution.has_accounting(),
            bytes,
        )
        .map_err(CompanionPreflightStop::Execution)?;
        execution
            .run_native_with_checkpoint(|context| {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                    context, bytes,
                )
            })
            .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?
            .0
            .map_err(|stop| {
                CompanionPreflightStop::Execution(
                    worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                )
            })?;
        let result = Self {
            subject: Arc::new(Subject {
                producer: publication.producer().to_owned(),
                family: publication.meaning().family.clone(),
                source: publication.meaning().source_scope,
                coordinate: publication.authority().publication_coordinate(),
                _memory: memory,
            }),
        };
        subject_capture_for_test::record(publication);
        Ok(result)
    }
    pub fn producer(&self) -> &str {
        &self.subject.producer
    }
    pub fn family(&self) -> &str {
        &self.subject.family
    }
    pub fn source(&self) -> worth_relational::facade::identity::EntityId {
        self.subject.source
    }
    /// Original publishing commit and route ordinal; descriptive, with no issuer custody.
    pub fn publication_coordinate(&self) -> (u64, usize) {
        self.subject.coordinate
    }
    #[doc(hidden)]
    pub fn observe_capture_for_test() -> subject_capture_for_test::Observation {
        subject_capture_for_test::Observation::install()
    }
}
impl std::fmt::Debug for WorthQueryHistoryPublicationCorrespondenceMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryPublicationCorrespondenceMismatch")
            .field("producer", &self.producer())
            .field("family", &self.family())
            .field("source", &self.source())
            .field("publication", &self.publication_coordinate())
            .finish()
    }
}
impl From<CompanionPreflightStop> for HistoricalTransportDenial {
    fn from(cause: CompanionPreflightStop) -> Self {
        Self::Lookup(cause)
    }
}
