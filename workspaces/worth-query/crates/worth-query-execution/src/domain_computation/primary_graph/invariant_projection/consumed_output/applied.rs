//! Attempt-local prerequisite custody. No Current or settlement exists before publication.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryOutputDemandKey,
    output_lineage::CoupledOutputCompletion,
    provider::{CoupledApplicationMember, WorthQueryPrimaryGraphApplicationAttempt},
};
use worth_relational::facade::{
    identity::EntityId,
    mvcc::RequiredOutputPublicationMeaning,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};
type Pin = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
struct AppliedConsumptionCells {
    supported: OnceLock<()>,
    settled: OnceLock<ConsumedOutputEvidence>,
    /// The chain's latest round this consumption settled to (R3), when the
    /// consumer was not re-run: its writer and meaning replace the consumed
    /// ones for final support and publication. Each later settlement of the
    /// same chain replaces it.
    replacement: std::sync::Mutex<Option<SettledReplacement>>,
}
#[derive(Clone)]
struct SettledReplacement {
    completion: CoupledOutputCompletion,
    meaning: RequiredOutputPublicationMeaning,
    writer: Pin,
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
    pub(in crate::domain_computation::primary_graph) fn capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
        producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
        role: &str,
        entity: EntityId,
        selected: &Pin,
    ) -> Result<Self, worth_execution::ExecutionRequestArrayDenial> {
        let preparation = attempt
            .coupled_preparation()
            .expect("actual appended writer");
        let mut cells = worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, 1)?;
        cells.push(AppliedConsumptionCells {
            supported: OnceLock::new(),
            settled: OnceLock::new(),
            replacement: std::sync::Mutex::new(None),
        })?;
        Ok(Self {
            completion: preparation.completion.clone(),
            producer: producer.to_owned(),
            meaning: meaning.clone(),
            role: role.to_owned(),
            entity,
            writer: preparation
                .after_own
                .as_ref()
                .expect("actual post-own image")
                .clone(),
            selected: selected.clone(),
            key: attempt.required_output_key(),
            cells: Arc::new(cells.seal()?),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn entity(&self) -> EntityId {
        self.entity
    }
    pub(super) fn same_selection(&self, other: &Self) -> bool {
        self.completion.same_writer(&other.completion)
            && self.meaning == other.meaning
            && self.role == other.role
            && self.entity == other.entity
            && self.selected.candidate_identity() == other.selected.candidate_identity()
    }
    /// Whether this consumption read route `meaning` of `producer`.
    pub(in crate::domain_computation::primary_graph) fn reads_route(
        &self,
        producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> bool {
        self.producer == producer && same_route(&self.meaning, meaning)
    }
    /// Whether the chain's later round in `attempt` publishes exactly what this
    /// consumption read: same producer, route, role and entity, and demand
    /// key (the Query half of R3 settlement). With `settle`, it becomes the
    /// consumption's replacement.
    pub(in crate::domain_computation::primary_graph) fn settles_to(
        &self,
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
        settle: bool,
    ) -> bool {
        let Some(preparation) = attempt.coupled_preparation() else {
            return false;
        };
        let (Some(outputs), Some(writer)) = (&preparation.outputs, &preparation.after_own) else {
            return false;
        };
        let same = outputs.producer == self.producer
            && same_route(&outputs.meaning, &self.meaning)
            && attempt
                .native_publication_roles()
                .zip(outputs.entities.iter())
                .any(|((role, _, _, _), entity)| role == self.role && *entity == self.entity)
            && attempt.required_output_key() == self.key;
        if same && settle {
            *self.replacement_cell() = Some(SettledReplacement {
                completion: preparation.completion.clone(),
                meaning: outputs.meaning.clone(),
                writer: writer.clone(),
            });
        }
        same
    }
    fn replacement_cell(&self) -> std::sync::MutexGuard<'_, Option<SettledReplacement>> {
        self.cells[0]
            .replacement
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn replacement(&self) -> Option<SettledReplacement> {
        self.replacement_cell().clone()
    }
    /// The completion this consumption settles against at publication: its
    /// settled replacement's, or the consumed writer's.
    pub(in crate::domain_computation::primary_graph) fn settling_completion(
        &self,
    ) -> CoupledOutputCompletion {
        self.replacement()
            .map_or_else(|| self.completion.clone(), |settled| settled.completion)
    }
    pub(in crate::domain_computation::primary_graph) fn require_support(
        &self,
        position: usize,
        members: &[CoupledApplicationMember],
        complete: &Pin,
    ) -> bool {
        let replacement = self.replacement();
        let (completion, meaning, writer) = match &replacement {
            Some(settled) => (&settled.completion, &settled.meaning, &settled.writer),
            None => (&self.completion, &self.meaning, &self.writer),
        };
        let Some(member) = members.iter().find(|member| {
            member
                .producer_route()
                .is_some_and(|(producer, route)| producer == self.producer && route == meaning)
        }) else {
            return false;
        };
        if complete.candidate_identity().proposal() != self.selected.candidate_identity().proposal()
        {
            return false;
        }
        // The consumed member precedes the consumer and lies in the prefix it
        // read; a settled replacement, appended after it, is admitted only by
        // its settlement proof.
        if replacement.is_none()
            && (member.position >= position
                || !self
                    .selected
                    .candidate_identity()
                    .includes_assembly_prefix(writer.candidate_identity()))
        {
            return false;
        }
        let attempt = member.extracted_attempt();
        let Some(preparation) = attempt.coupled_preparation() else {
            return false;
        };
        let Some(outputs) = &preparation.outputs else {
            return false;
        };
        let matches = completion.same_writer(&preparation.completion)
            && preparation
                .after_own
                .as_ref()
                .is_some_and(|pin| pin.candidate_identity() == writer.candidate_identity())
            && outputs.meaning == *meaning
            && outputs.producer == self.producer
            && attempt
                .native_publication_roles()
                .zip(outputs.entities.iter())
                .any(|((role, _, _, _), entity)| role == self.role && *entity == self.entity)
            && attempt.required_output_key() == self.key;
        if matches {
            let _ = self.cells[0].supported.set(());
        }
        matches
    }
    pub(in crate::domain_computation::primary_graph) fn prerequisite_key(
        &self,
    ) -> Option<&Arc<WorthQueryOutputDemandKey>> {
        self.cells[0].supported.get()?;
        self.key.as_ref()
    }
    pub(in crate::domain_computation::primary_graph) fn settle(
        &self,
        after: &PositionedRelationalSnapshot,
    ) {
        assert!(
            self.cells[0].supported.get().is_some(),
            "final support precedes publication"
        );
        let completion = self.settling_completion();
        let settled = completion.settled_consumption(after.clone(), &self.role, self.entity);
        assert!(
            self.cells[0].settled.set(settled).is_ok(),
            "one canonical publication join"
        );
    }
    pub(super) fn is_settled(&self) -> bool {
        self.cells[0].settled.get().is_some()
    }
    pub(super) fn recorded(&self) -> &RecordedConsumedOutput {
        self.cells[0]
            .settled
            .get()
            .expect("only committed readers resolve settled custody")
            .recorded()
    }
}

/// The same owned route: family, source scope and source partition.
fn same_route(
    left: &RequiredOutputPublicationMeaning,
    right: &RequiredOutputPublicationMeaning,
) -> bool {
    left.family == right.family
        && left.source_scope == right.source_scope
        && left.source_partition == right.source_partition
}
