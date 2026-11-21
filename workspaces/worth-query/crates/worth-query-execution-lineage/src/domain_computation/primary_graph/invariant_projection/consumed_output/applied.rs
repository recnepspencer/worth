//! Attempt-local prerequisite custody. No Current or settlement exists before publication.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryOutputDemandKey, output_lineage::CoupledOutputCompletion,
};
use crate::ports::{ApplicationAttemptPort, CoupledMemberPort};
use worth_relational::facade::{
    identity::EntityId,
    mvcc::RequiredOutputPublicationMeaning,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};
type Pin = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
struct AppliedConsumptionCells {
    supported: OnceLock<()>,
    settled: OnceLock<ConsumedOutputEvidence>,
}
#[derive(Clone)]
pub struct AppliedConsumedOutput {
    completion: CoupledOutputCompletion,
    producer: String,
    meaning: RequiredOutputPublicationMeaning,
    role: String,
    entity: EntityId,
    writer: Pin,
    selected: Pin,
    key: Option<Arc<WorthQueryOutputDemandKey>>,
    cells: Arc<worth_execution::ExecutionArray<AppliedConsumptionCells>>,
}
impl std::fmt::Debug for AppliedConsumedOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppliedConsumedOutput")
            .field("meaning", &self.meaning)
            .field("role", &self.role)
            .field("entity", &self.entity)
            .finish_non_exhaustive()
    }
}
impl AppliedConsumedOutput {
    pub fn capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        attempt: &dyn ApplicationAttemptPort,
        producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
        role: &str,
        entity: EntityId,
        selected: &Pin,
    ) -> Result<Self, worth_execution::ExecutionRequestArrayDenial> {
        let preparation = attempt
            .coupled_preparation_view()
            .expect("actual appended writer");
        let mut cells = worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, 1)?;
        cells.push(AppliedConsumptionCells {
            supported: OnceLock::new(),
            settled: OnceLock::new(),
        })?;
        Ok(Self {
            completion: preparation.completion.clone(),
            producer: producer.to_owned(),
            meaning: meaning.clone(),
            role: role.to_owned(),
            entity,
            writer: preparation
                .after_own
                .expect("actual post-own image")
                .clone(),
            selected: selected.clone(),
            key: attempt.required_output_key(),
            cells: Arc::new(cells.seal()?),
        })
    }
    pub fn entity(&self) -> EntityId {
        self.entity
    }
    pub fn same_selection(&self, other: &Self) -> bool {
        self.completion.same_writer(&other.completion)
            && self.meaning == other.meaning
            && self.role == other.role
            && self.entity == other.entity
            && self.selected.candidate_identity() == other.selected.candidate_identity()
    }
    pub fn require_support<M: CoupledMemberPort>(
        &self,
        position: usize,
        members: &[M],
        complete: &Pin,
    ) -> bool {
        let Some(member) = members.iter().find(|member| {
            member.producer_route().is_some_and(|(producer, meaning)| {
                producer == self.producer && meaning == &self.meaning
            })
        }) else {
            return false;
        };
        if member.position() >= position
            || complete.candidate_identity().proposal()
                != self.selected.candidate_identity().proposal()
            || !self
                .selected
                .candidate_identity()
                .includes_assembly_prefix(self.writer.candidate_identity())
        {
            return false;
        }
        let attempt = member.extracted_attempt();
        let Some(preparation) = attempt.coupled_preparation_view() else {
            return false;
        };
        let (Some(output_meaning), Some(output_producer)) =
            (preparation.output_meaning, preparation.output_producer)
        else {
            return false;
        };
        let matches = self.completion.same_writer(preparation.completion)
            && preparation
                .after_own
                .is_some_and(|pin| pin.candidate_identity() == self.writer.candidate_identity())
            && output_meaning == &self.meaning
            && output_producer == self.producer
            && attempt.has_native_publication_role_entity(&self.role, self.entity)
            && attempt.required_output_key() == self.key;
        if matches {
            let _ = self.cells[0].supported.set(());
        }
        matches
    }
    pub fn prerequisite_key(&self) -> Option<&Arc<WorthQueryOutputDemandKey>> {
        self.cells[0].supported.get()?;
        self.key.as_ref()
    }
    pub fn settle(&self, after: &PositionedRelationalSnapshot) {
        assert!(
            self.cells[0].supported.get().is_some(),
            "final support precedes publication"
        );
        let settled = self
            .completion
            .settled_consumption(after.clone(), &self.role, self.entity);
        assert!(
            self.cells[0].settled.set(settled).is_ok(),
            "one canonical publication join"
        );
    }
    pub fn is_settled(&self) -> bool {
        self.cells[0].settled.get().is_some()
    }
    pub fn recorded(&self) -> &RecordedConsumedOutput {
        self.cells[0]
            .settled
            .get()
            .expect("only committed readers resolve settled custody")
            .recorded()
    }
}
