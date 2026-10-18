use super::comparison::{compare, ComparisonFailure, ComparisonMismatch, ObservedSupplyChainState};
use super::delta::DeltaId;
use super::expected_digest::canonical_bytes;
use super::expected_observation::ExpectedSupplyChainObservation;
use super::oracle::{apply, AncestryError, OracleApplicationError};
use super::scale::{ScaleName, SupplyChainScale};
use super::scenarios::{BaselineName, SupplyChainBaseline};
use super::schema::RelationEdge;
use super::semantic_key::{Anchor, BranchLabel, EntityKey, EntityKind, RelationKey, RelationKind};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MutationId {
    MissingWrite,
    SiblingLeak,
    FloatingBranch,
    WrongAncestry,
    DuplicateRelation,
    IllegalEndpoint,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum MutationOperation {
    RemoveEntity,
    CopySiblingEntity,
    SelectOperatingBranch,
    ReplaceParent,
    DuplicateRelation,
    RepointEndpoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TraceReplayError {
    UnsupportedVersion(u16),
    SeedMismatch {
        expected: u64,
        observed: u64,
    },
    InvalidMutationPair(MutationId, MutationOperation),
    InvalidBranch(AncestryError),
    DeltaApplication {
        step: usize,
        delta: DeltaId,
        source: OracleApplicationError,
    },
    RecordedInputMismatch {
        expected: Vec<u8>,
        observed: Vec<u8>,
    },
    RecordedRelationVectorMismatch {
        expected: Vec<u8>,
        observed: Vec<u8>,
    },
    MutatedInputMismatch {
        expected: Vec<u8>,
        observed: Vec<u8>,
    },
    ForgedFirstDivergence {
        expected: ComparisonMismatch,
        observed: ComparisonMismatch,
    },
    MissingDivergence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TraceReplayResult {
    pub(crate) replayed_trace: SemanticTrace,
    pub(crate) observation: ExpectedSupplyChainObservation,
    pub(crate) comparison: ComparisonFailure,
    pub(crate) first_divergence: ComparisonMismatch,
    pub(crate) relation_vector_input: Vec<u8>,
    pub(crate) mutated_input: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticTrace {
    pub(crate) version: u16,
    pub(crate) profile: ScaleName,
    pub(crate) seed: u64,
    pub(crate) baseline: BaselineName,
    pub(crate) branch: BranchLabel,
    pub(crate) deltas: Vec<DeltaId>,
    pub(crate) mutation: Option<(MutationId, MutationOperation, Vec<u8>)>,
    pub(crate) recorded_relation_vector: Option<Vec<u8>>,
    pub(crate) mutated_input: Option<Vec<u8>>,
    pub(crate) first_divergence: Option<ComparisonMismatch>,
}

impl SemanticTrace {
    pub(crate) fn new(
        scale: SupplyChainScale,
        baseline: BaselineName,
        branch: BranchLabel,
        deltas: Vec<DeltaId>,
    ) -> Self {
        Self {
            version: 1,
            profile: scale.name,
            seed: scale.seed,
            baseline,
            branch,
            deltas,
            mutation: None,
            recorded_relation_vector: None,
            mutated_input: None,
            first_divergence: None,
        }
    }

    pub(crate) fn with_mutation(
        mut self,
        id: MutationId,
        operation: MutationOperation,
        observation: &ExpectedSupplyChainObservation,
    ) -> Self {
        let relation_vector = observation.relations.values().copied().collect::<Vec<_>>();
        self.mutation = Some((id, operation, canonical_bytes(observation)));
        self.recorded_relation_vector = Some(relation_vector_bytes(&relation_vector));
        self
    }

    pub(crate) fn step_count(&self) -> usize {
        1 + self.deltas.len() + usize::from(self.mutation.is_some())
    }

    pub(crate) fn replay(&self) -> Result<TraceReplayResult, TraceReplayError> {
        if self.version != 1 {
            return Err(TraceReplayError::UnsupportedVersion(self.version));
        }
        let scale = scale_for(self.profile);
        if scale.seed != self.seed {
            return Err(TraceReplayError::SeedMismatch {
                expected: scale.seed,
                observed: self.seed,
            });
        }
        let Some((mutation, operation, recorded_input)) = &self.mutation else {
            return Err(TraceReplayError::MissingDivergence);
        };
        if !mutation_matches(*mutation, *operation) {
            return Err(TraceReplayError::InvalidMutationPair(*mutation, *operation));
        }
        let baseline = baseline_for(self.baseline, scale);
        let mut branch = if self.branch == BranchLabel::Operating {
            baseline.branch.clone()
        } else {
            baseline
                .branch
                .fork(self.branch, BranchLabel::Operating)
                .map_err(TraceReplayError::InvalidBranch)?
        };
        for (step, delta) in self.deltas.iter().copied().enumerate() {
            branch =
                apply(&branch, delta).map_err(|source| TraceReplayError::DeltaApplication {
                    step,
                    delta,
                    source,
                })?;
        }
        let expected = ExpectedSupplyChainObservation::from_branch(&branch);
        let reconstructed_input = canonical_bytes(&expected);
        if reconstructed_input != *recorded_input {
            return Err(TraceReplayError::RecordedInputMismatch {
                expected: recorded_input.clone(),
                observed: reconstructed_input,
            });
        }
        let relation_vector = expected.relations.values().copied().collect::<Vec<_>>();
        let relation_vector_input = relation_vector_bytes(&relation_vector);
        if let Some(recorded) = &self.recorded_relation_vector
            && recorded != &relation_vector_input
        {
            return Err(TraceReplayError::RecordedRelationVectorMismatch {
                expected: recorded.clone(),
                observed: relation_vector_input,
            });
        }
        let mut observed = ObservedSupplyChainState::from_expected(&expected);
        apply_mutation(&mut observed, *mutation, &baseline)?;
        let comparison = match compare(&expected, &observed) {
            Ok(()) => return Err(TraceReplayError::MissingDivergence),
            Err(failure) => failure,
        };
        let first_divergence = comparison.mismatch.clone();
        if let Some(recorded) = &self.first_divergence
            && recorded != &first_divergence
        {
            return Err(TraceReplayError::ForgedFirstDivergence {
                expected: first_divergence,
                observed: recorded.clone(),
            });
        }
        let mutated_input = observed_canonical_bytes(&observed);
        if let Some(recorded) = &self.mutated_input
            && recorded != &mutated_input
        {
            return Err(TraceReplayError::MutatedInputMismatch {
                expected: recorded.clone(),
                observed: mutated_input,
            });
        }
        let mut replayed_trace = self.clone();
        replayed_trace.mutated_input = Some(mutated_input.clone());
        replayed_trace.first_divergence = Some(first_divergence.clone());
        Ok(TraceReplayResult {
            replayed_trace,
            observation: expected,
            comparison,
            first_divergence,
            relation_vector_input,
            mutated_input,
        })
    }

    pub(crate) fn replay_fingerprint(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.push(self.profile as u8);
        bytes.extend_from_slice(&self.seed.to_be_bytes());
        bytes.push(self.baseline as u8);
        bytes.push(self.branch as u8);
        for delta in &self.deltas {
            bytes.push(*delta as u8);
        }
        if let Some((id, operation, input)) = &self.mutation {
            bytes.push(*id as u8);
            bytes.push(*operation as u8);
            bytes.extend_from_slice(&(input.len() as u32).to_be_bytes());
            bytes.extend_from_slice(input);
        }
        if let Some(input) = &self.recorded_relation_vector {
            bytes.extend_from_slice(b"\0supply-chain-recorded-relation-vector-v1\0");
            bytes.extend_from_slice(&(input.len() as u32).to_be_bytes());
            bytes.extend_from_slice(input);
        }
        if let Some(input) = &self.mutated_input {
            bytes.extend_from_slice(&(input.len() as u32).to_be_bytes());
            bytes.extend_from_slice(input);
        }
        bytes
    }
}

fn scale_for(profile: ScaleName) -> SupplyChainScale {
    match profile {
        ScaleName::Court => SupplyChainScale::court(),
        ScaleName::Standard => SupplyChainScale::standard(),
        ScaleName::Scale => SupplyChainScale::scale(),
    }
}

fn baseline_for(name: BaselineName, scale: SupplyChainScale) -> SupplyChainBaseline {
    match name {
        BaselineName::EmptyInstallation => SupplyChainBaseline::empty(scale),
        BaselineName::Operating => SupplyChainBaseline::operating(scale),
        BaselineName::ContestedPlanning => SupplyChainBaseline::contested(scale),
        BaselineName::RetentionPressure => SupplyChainBaseline::retention_pressure(scale),
        BaselineName::VersionBoundary => SupplyChainBaseline::version_boundary(scale),
    }
}

fn mutation_matches(id: MutationId, operation: MutationOperation) -> bool {
    matches!(
        (id, operation),
        (MutationId::MissingWrite, MutationOperation::RemoveEntity)
            | (
                MutationId::SiblingLeak,
                MutationOperation::CopySiblingEntity
            )
            | (
                MutationId::FloatingBranch,
                MutationOperation::SelectOperatingBranch
            )
            | (MutationId::WrongAncestry, MutationOperation::ReplaceParent)
            | (
                MutationId::DuplicateRelation,
                MutationOperation::DuplicateRelation
            )
            | (
                MutationId::IllegalEndpoint,
                MutationOperation::RepointEndpoint
            )
    )
}

fn apply_mutation(
    observed: &mut ObservedSupplyChainState,
    id: MutationId,
    baseline: &SupplyChainBaseline,
) -> Result<(), TraceReplayError> {
    match id {
        MutationId::MissingWrite => observed.remove_entity(Anchor::AuroraEastbound.entity()),
        MutationId::SiblingLeak => {
            let sibling = baseline
                .branch
                .fork(BranchLabel::CompetingArrival, BranchLabel::Operating)
                .map_err(TraceReplayError::InvalidBranch)?;
            let sibling = apply(&sibling, DeltaId::CompetingAuroraArrival).map_err(|source| {
                TraceReplayError::DeltaApplication {
                    step: 0,
                    delta: DeltaId::CompetingAuroraArrival,
                    source,
                }
            })?;
            if let Some(value) = sibling.state.entity(Anchor::AuroraEastbound.entity()) {
                observed.set_entity(Anchor::AuroraEastbound.entity(), value.clone());
            }
        }
        MutationId::FloatingBranch => observed.set_branch(BranchLabel::Operating),
        MutationId::WrongAncestry => observed.set_parent(Some(BranchLabel::Customs)),
        MutationId::DuplicateRelation => {
            observed.duplicate_relation(RelationKey::new(RelationKind::CallAtPort, 1));
        }
        MutationId::IllegalEndpoint => observed.repoint_relation(
            RelationKey::new(RelationKind::CallAtPort, 1),
            EntityKey::new(EntityKind::Port, u32::MAX),
        ),
    }
    Ok(())
}

fn observed_as_expected(observed: &ObservedSupplyChainState) -> ExpectedSupplyChainObservation {
    ExpectedSupplyChainObservation {
        schema: super::schema::SupplyChainSchema::canonical(observed.schema),
        entities: observed.entities.clone(),
        relations: observed.relations.clone(),
        absent_entities: observed.absent_entities.clone(),
        absent_relations: observed.absent_relations.clone(),
        ancestry: super::oracle::OracleAncestry {
            branch: observed.branch,
            parent: observed.parent,
            lineage: observed.lineage.clone(),
            accepted: observed.accepted.clone(),
            history: observed.history.clone(),
        },
    }
}

fn observed_canonical_bytes(observed: &ObservedSupplyChainState) -> Vec<u8> {
    let mut bytes = canonical_bytes(&observed_as_expected(observed));
    bytes.extend_from_slice(b"\0supply-chain-relation-vector-v1\0");
    bytes.extend_from_slice(&relation_vector_bytes(&observed.relation_vector));
    bytes
}

fn relation_vector_bytes(edges: &[RelationEdge]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(edges.len() as u32).to_be_bytes());
    for edge in edges {
        bytes.push(edge.key.kind as u8);
        bytes.extend_from_slice(&edge.key.ordinal.to_be_bytes());
        bytes.push(edge.source.kind as u8);
        bytes.extend_from_slice(&edge.source.ordinal.to_be_bytes());
        bytes.push(edge.target.kind as u8);
        bytes.extend_from_slice(&edge.target.ordinal.to_be_bytes());
    }
    bytes
}
