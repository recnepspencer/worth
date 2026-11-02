//! Native's finite initialization artifact has one actual anchored durable step.
use super::StoreJournalMedium;
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    DurablePublicationFate as Fate, RelationalInitialInputsPublicationDetail as Detail,
    RelationalSourceInitialInputsArtifact,
};
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure as Failure,
    PhysicalMutationDeadline,
};

impl StoreJournalMedium {
    pub(super) fn publish_original_initial_inputs(
        &self,
        artifact: &RelationalSourceInitialInputsArtifact,
        request: &ExecutionRequest<'_, '_>,
    ) -> Fate<Detail> {
        if let Err(cause) = request.consult() {
            return Fate::ProvenNoEffect {
                detail: Detail::Request(cause.into()),
            };
        }
        let records = match self.serving.anchored_records() {
            Ok(records) => records,
            Err(cause) => {
                return Fate::ProvenNoEffect {
                    detail: Detail::Physical(format!("initial inputs unavailable: {cause:?}")),
                }
            }
        };
        let milliseconds = request.deadline().map_or(30_000, |deadline| {
            deadline
                .saturating_duration_since(std::time::Instant::now())
                .as_millis()
                .min(30_000) as u64
        });
        let Some(deadline) = PhysicalMutationDeadline::after_milliseconds(milliseconds) else {
            return Fate::ProvenNoEffect {
                detail: Detail::Physical(
                    "initial input deadline is below physical resolution".into(),
                ),
            };
        };
        let publication = match AnchoredRecordPublication::new_for_request(
            AnchoredRecordName::new("relational.initial-inputs").expect("static Native anchor"),
            artifact.revision() - 1,
            artifact.as_bytes(),
            self.placement,
            deadline,
            request,
        ) {
            Ok(publication) => publication,
            Err(cause) => {
                return Fate::ProvenNoEffect {
                    detail: Detail::Request(cause.into()),
                }
            }
        };
        match records.publish(publication) {
            Ok(_) => Fate::Committed,
            Err(cause) => classify(cause),
        }
    }
}
fn classify(cause: Failure) -> Fate<Detail> {
    match cause.into_publication_fate() {
        Fate::Committed => unreachable!("a publication failure cannot report commitment"),
        Fate::ProvenNoEffect { detail } => Fate::ProvenNoEffect {
            detail: Detail::Physical(format!("{detail:?}")),
        },
        Fate::Indeterminate { detail } => Fate::Indeterminate {
            detail: Detail::Physical(format!("{detail:?}")),
        },
    }
}
