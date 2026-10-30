pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison::compare, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonFailure, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_digest::canonical_bytes};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError, crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName, crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MutationId {
    MissingWrite,
    SiblingLeak,
    FloatingBranch,
    WrongAncestry,
    DuplicateRelation,
    IllegalEndpoint,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MutationOperation {
    RemoveEntity,
    CopySiblingEntity,
    SelectOperatingBranch,
    ReplaceParent,
    DuplicateRelation,
    RepointEndpoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceReplayError {
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
pub struct TraceReplayResult {
    pub replayed_trace: SemanticTrace,
    pub observation: ExpectedSupplyChainObservation,
    pub comparison: ComparisonFailure,
    pub first_divergence: ComparisonMismatch,
    pub relation_vector_input: Vec<u8>,
    pub mutated_input: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticTrace {
    pub version: u16,
    pub profile: ScaleName,
    pub seed: u64,
    pub baseline: BaselineName,
    pub branch: BranchLabel,
    pub deltas: Vec<DeltaId>,
    pub mutation: Option<(MutationId, MutationOperation, Vec<u8>)>,
    pub recorded_relation_vector: Option<Vec<u8>>,
    pub mutated_input: Option<Vec<u8>>,
    pub first_divergence: Option<ComparisonMismatch>,
}

impl SemanticTrace {
    pub fn new(
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

    pub fn with_mutation(
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

    pub fn step_count(&self) -> usize {
        1 + self.deltas.len() + usize::from(self.mutation.is_some())
    }

    pub fn replay(&self) -> Result<TraceReplayResult, TraceReplayError> {
        if self.version != 1 {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::UnsupportedVersion(self.version));
        }
        let scale = scale_for(self.profile);
        if scale.seed != self.seed {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::SeedMismatch {
                expected: scale.seed,
                observed: self.seed,
            });
        }
        let Some((mutation, operation, recorded_input)) = &self.mutation else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::MissingDivergence);
        };
        if !mutation_matches(*mutation, *operation) {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::InvalidMutationPair(*mutation, *operation));
        }
        let baseline = baseline_for(self.baseline, scale);
        let mut branch = if self.branch == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating {
            baseline.branch.clone()
        } else {
            baseline
                .branch
                .fork(self.branch, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .map_err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::InvalidBranch)?
        };
        for (step, delta) in self.deltas.iter().copied().enumerate() {
            branch =
                apply(&branch, delta).map_err(|source| crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::DeltaApplication {
                    step,
                    delta,
                    source,
                })?;
        }
        let expected = crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&branch);
        let reconstructed_input = canonical_bytes(&expected);
        if reconstructed_input != *recorded_input {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::RecordedInputMismatch {
                expected: recorded_input.clone(),
                observed: reconstructed_input,
            });
        }
        let relation_vector = expected.relations.values().copied().collect::<Vec<_>>();
        let relation_vector_input = relation_vector_bytes(&relation_vector);
        if let Some(recorded) = &self.recorded_relation_vector
            && recorded != &relation_vector_input
        {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::RecordedRelationVectorMismatch {
                expected: recorded.clone(),
                observed: relation_vector_input,
            });
        }
        let mut observed = crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
        apply_mutation(&mut observed, *mutation, &baseline)?;
        let comparison = match compare(&expected, &observed) {
            Ok(()) => return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::MissingDivergence),
            Err(failure) => failure,
        };
        let first_divergence = comparison.mismatch.clone();
        if let Some(recorded) = &self.first_divergence
            && recorded != &first_divergence
        {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::ForgedFirstDivergence {
                expected: first_divergence,
                observed: recorded.clone(),
            });
        }
        let mutated_input = observed_canonical_bytes(&observed);
        if let Some(recorded) = &self.mutated_input
            && recorded != &mutated_input
        {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::MutatedInputMismatch {
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

    pub fn replay_fingerprint(&self) -> Vec<u8> {
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

pub fn scale_for(profile: ScaleName) -> SupplyChainScale {
    match profile {
        crate::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Court => crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court(),
        crate::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Standard => crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard(),
        crate::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Scale => crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::scale(),
    }
}

pub fn baseline_for(name: BaselineName, scale: SupplyChainScale) -> SupplyChainBaseline {
    match name {
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::EmptyInstallation => crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::empty(scale),
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating => crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale),
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::ContestedPlanning => crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::contested(scale),
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::RetentionPressure => crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::retention_pressure(scale),
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::VersionBoundary => crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::version_boundary(scale),
    }
}

pub fn mutation_matches(id: MutationId, operation: MutationOperation) -> bool {
    matches!(
        (id, operation),
        (crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite, crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RemoveEntity)
            | (
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::SiblingLeak,
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::CopySiblingEntity
            )
            | (
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::FloatingBranch,
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::SelectOperatingBranch
            )
            | (crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::WrongAncestry, crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::ReplaceParent)
            | (
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::DuplicateRelation,
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::DuplicateRelation
            )
            | (
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::IllegalEndpoint,
                crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationOperation::RepointEndpoint
            )
    )
}

pub fn apply_mutation(
    observed: &mut ObservedSupplyChainState,
    id: MutationId,
    baseline: &SupplyChainBaseline,
) -> Result<(), TraceReplayError> {
    match id {
        crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::MissingWrite => observed.remove_entity(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity()),
        crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::SiblingLeak => {
            let sibling = baseline
                .branch
                .fork(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::CompetingArrival, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
                .map_err(crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::InvalidBranch)?;
            let sibling = apply(&sibling, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival).map_err(|source| {
                crate::relational_certification::ordinary_context::world::supply_chain::trace::TraceReplayError::DeltaApplication {
                    step: 0,
                    delta: crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival,
                    source,
                }
            })?;
            if let Some(value) = sibling.state.entity(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity()) {
                observed.set_entity(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity(), value.clone());
            }
        }
        crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::FloatingBranch => observed.set_branch(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating),
        crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::WrongAncestry => observed.set_parent(Some(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs)),
        crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::DuplicateRelation => {
            observed.duplicate_relation(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::trace::MutationId::IllegalEndpoint => observed.repoint_relation(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1),
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, u32::MAX),
        ),
    }
    Ok(())
}

pub fn observed_as_expected(observed: &ObservedSupplyChainState) -> ExpectedSupplyChainObservation {
    ExpectedSupplyChainObservation {
        schema: crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema::canonical(observed.schema),
        entities: observed.entities.clone(),
        relations: observed.relations.clone(),
        absent_entities: observed.absent_entities.clone(),
        absent_relations: observed.absent_relations.clone(),
        ancestry: crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleAncestry {
            branch: observed.branch,
            parent: observed.parent,
            lineage: observed.lineage.clone(),
            accepted: observed.accepted.clone(),
            history: observed.history.clone(),
        },
    }
}

pub fn observed_canonical_bytes(observed: &ObservedSupplyChainState) -> Vec<u8> {
    let mut bytes = canonical_bytes(&observed_as_expected(observed));
    bytes.extend_from_slice(b"\0supply-chain-relation-vector-v1\0");
    bytes.extend_from_slice(&relation_vector_bytes(&observed.relation_vector));
    bytes
}

pub fn relation_vector_bytes(edges: &[RelationEdge]) -> Vec<u8> {
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
