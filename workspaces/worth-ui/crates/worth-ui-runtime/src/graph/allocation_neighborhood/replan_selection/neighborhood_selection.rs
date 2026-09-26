use super::UiReplanLocalityProof;
use crate::evidence::UiAllocationNeighborhoodIdentity;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiReplanWidenReason {
    ConstraintPropagationCrossing,
    SharedAncestorRequirement,
    PortalLayerSpan,
    ScrollOwnerContainment,
    PolicyMergeEscalation,
    ViewportShellRequirement,
    MeasurementBasisReach,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiReplanRootPosture {
    NotRoot,
    RootPrimary,
    CountedRootWiden { reason: UiReplanWidenReason },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiReplanOverlapDisposition {
    Singleton,
    PairwiseDisjoint,
    ContainmentMerged,
    ContainmentSuperseded,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAdmittedReplanNeighborhood {
    identity: UiAllocationNeighborhoodIdentity,
    locality: UiReplanLocalityProof,
    widen_reason: Option<UiReplanWidenReason>,
    allocation_plan: crate::runtime::UiAdmittedAllocationPlanReference,
    neighborhood_footprint: std::rc::Rc<super::super::UiGraphNeighborhoodFootprint>,
    replacement_impact: Option<std::rc::Rc<crate::runtime::WorthUiReplacementImpactClassification>>,
    impact_narrowing: Option<std::rc::Rc<crate::runtime::WorthUiRuntimeImpactNarrowing>>,
    root_target: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAdmittedReplanNeighborhoodSet {
    transaction_basis: UiGraphReplanTransactionBasis,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiGraphReplanTransactionBasis {
    frame_identity: crate::runtime::UiAllocationFramePlanIdentity,
    ordered: Box<[UiAdmittedReplanNeighborhood]>,
    root_posture: UiReplanRootPosture,
    overlap_disposition: UiReplanOverlapDisposition,
    counters: UiReplanNeighborhoodSelectionCounters,
    consequences: super::UiGraphReplanConsequences,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiReplanNeighborhoodSelectionCounters {
    invalidation_visits: u16,
    locality_proofs: u16,
    set_cardinality: u16,
    root_widen_attempts: u16,
    graph_index_probes: u16,
    replacement_consequence_reads: u16,
    overlap_index_probes: u16,
    merged_neighborhoods: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiReplanLocalityDenial {
    EmptyInvalidationSet,
    ConflictingNeighborhoodForTarget,
    ForbiddenRootFallback,
    UnsupportedWideningFamily,
    MissingAdmittedCandidate,
    IncompleteReplacementLineage,
    AdmittedGenerationSetChanged,
    OverlappingNeighborhoods { left: u16, right: u16 },
    OverlappingNeighborhoodSupersessionRequired,
    EmptyScrollConsequence,
    ContradictoryScrollConsequence,
    QueryMeasurementSuccessorDenied,
    HostMeasurementSuccessorDenied,
    CounterExhausted,
}

impl UiAdmittedReplanNeighborhood {
    pub(in crate::graph::allocation_neighborhood) fn primary(
        target: &crate::graph::UiAdmittedAllocationInvalidationTarget,
    ) -> Result<Self, UiReplanLocalityDenial> {
        let replacement_lineage = super::replan_locality::retain_replacement_lineage(target)?;
        Ok(Self {
            identity: target.neighborhood_identity().clone(),
            locality: UiReplanLocalityProof::from_target(target)?,
            widen_reason: None,
            allocation_plan: target
                .allocation_plan()
                .cloned()
                .ok_or(UiReplanLocalityDenial::MissingAdmittedCandidate)?,
            neighborhood_footprint: target.neighborhood_footprint(),
            replacement_impact: replacement_lineage
                .as_ref()
                .map(|(impact, _)| impact.clone()),
            impact_narrowing: replacement_lineage.map(|(_, narrowing)| narrowing),
            root_target: target.disposition()
                == crate::graph::UiGraphReplanTargetDisposition::RootPrimaryEligible,
        })
    }
    pub(in crate::graph::allocation_neighborhood) fn widened(
        target: &crate::graph::UiAdmittedAllocationInvalidationTarget,
        reason: UiReplanWidenReason,
    ) -> Result<Self, UiReplanLocalityDenial> {
        let replacement_lineage = super::replan_locality::retain_replacement_lineage(target)?;
        Ok(Self {
            identity: target.neighborhood_identity().clone(),
            locality: UiReplanLocalityProof::from_target(target)?,
            widen_reason: Some(reason),
            allocation_plan: target
                .allocation_plan()
                .cloned()
                .ok_or(UiReplanLocalityDenial::MissingAdmittedCandidate)?,
            neighborhood_footprint: target.neighborhood_footprint(),
            replacement_impact: replacement_lineage
                .as_ref()
                .map(|(impact, _)| impact.clone()),
            impact_narrowing: replacement_lineage.map(|(_, narrowing)| narrowing),
            root_target: target.disposition()
                == crate::graph::UiGraphReplanTargetDisposition::RootPrimaryEligible,
        })
    }
    pub fn identity(&self) -> &UiAllocationNeighborhoodIdentity {
        &self.identity
    }
    pub fn locality(&self) -> &UiReplanLocalityProof {
        &self.locality
    }
    pub fn widen_reason(&self) -> Option<UiReplanWidenReason> {
        self.widen_reason
    }
    pub(crate) fn allocation_candidate(&self) -> &crate::runtime::UiAllocationCandidate {
        self.allocation_plan.candidate()
    }
    pub(crate) fn replacement_lineage(
        &self,
    ) -> Option<(
        std::rc::Rc<crate::runtime::WorthUiReplacementImpactClassification>,
        std::rc::Rc<crate::runtime::WorthUiRuntimeImpactNarrowing>,
    )> {
        self.replacement_impact
            .as_ref()
            .zip(self.impact_narrowing.as_ref())
            .map(|(impact, narrowing)| (std::rc::Rc::clone(impact), std::rc::Rc::clone(narrowing)))
    }
    pub fn planning_identity_digest(&self) -> u64 {
        self.allocation_plan.planning_identity_digest()
    }
    pub(crate) fn is_root_target(&self) -> bool {
        self.root_target
    }
    pub(crate) fn neighborhood_members(&self) -> &[crate::graph::UiGraphNodeIdentity] {
        self.neighborhood_footprint.members()
    }
    pub(crate) fn generation_key(&self) -> crate::graph::UiReplanGenerationKey {
        self.allocation_plan.generation_key()
    }
}

impl UiAdmittedReplanNeighborhoodSet {
    pub(in crate::graph::allocation_neighborhood) fn new(
        frame_identity: &crate::runtime::UiAllocationFramePlanIdentity,
        ordered: Vec<UiAdmittedReplanNeighborhood>,
        root_posture: UiReplanRootPosture,
        overlap_disposition: UiReplanOverlapDisposition,
        counters: UiReplanNeighborhoodSelectionCounters,
        consequences: super::UiGraphReplanConsequences,
    ) -> Self {
        let transaction_basis = UiGraphReplanTransactionBasis::seal(
            frame_identity,
            ordered,
            root_posture,
            overlap_disposition,
            counters,
            consequences,
        );
        Self { transaction_basis }
    }
    pub fn frame_epoch(&self) -> crate::runtime::UiAllocationFrameEpoch {
        self.transaction_basis.frame_epoch()
    }
    pub fn policy(&self) -> crate::runtime::UiResolvedAllocationStreamPolicy {
        self.transaction_basis.policy()
    }
    pub fn ordered_neighborhoods(&self) -> &[UiAdmittedReplanNeighborhood] {
        &self.transaction_basis.ordered
    }
    /// How many neighborhoods the selection orders; sealing bounds a
    /// selection to what `u16` counts.
    pub fn neighborhood_count(&self) -> u16 {
        u16::try_from(self.transaction_basis.ordered.len())
            .expect("sealing bounds a selection to u16 neighborhoods")
    }
    /// The ordered neighborhoods, each with its ordinal in the selection.
    pub(crate) fn ordinal_neighborhoods(
        &self,
    ) -> impl Iterator<Item = (u16, &UiAdmittedReplanNeighborhood)> {
        (0..self.neighborhood_count()).zip(self.transaction_basis.ordered.iter())
    }
    pub fn primary(&self) -> &UiAdmittedReplanNeighborhood {
        &self.transaction_basis.ordered[0]
    }
    pub fn root_posture(&self) -> UiReplanRootPosture {
        self.transaction_basis.root_posture
    }
    pub fn overlap_disposition(&self) -> UiReplanOverlapDisposition {
        self.transaction_basis.overlap_disposition
    }
    pub fn counters(&self) -> UiReplanNeighborhoodSelectionCounters {
        self.transaction_basis.counters
    }
    pub(crate) fn transaction_basis(&self) -> &UiGraphReplanTransactionBasis {
        &self.transaction_basis
    }
}

impl UiGraphReplanTransactionBasis {
    fn seal(
        frame_identity: &crate::runtime::UiAllocationFramePlanIdentity,
        ordered: Vec<UiAdmittedReplanNeighborhood>,
        root_posture: UiReplanRootPosture,
        overlap_disposition: UiReplanOverlapDisposition,
        counters: UiReplanNeighborhoodSelectionCounters,
        consequences: super::UiGraphReplanConsequences,
    ) -> Self {
        Self {
            frame_identity: frame_identity.clone(),
            ordered: ordered.into_boxed_slice(),
            root_posture,
            overlap_disposition,
            counters,
            consequences,
        }
    }
    pub(crate) fn primary_neighborhood(&self) -> &UiAllocationNeighborhoodIdentity {
        self.ordered[0].identity()
    }
    pub(crate) fn frame_ingress_keys(&self) -> &[crate::runtime::UiAllocationFrameIngressKey] {
        self.frame_identity.ingress_keys()
    }
    pub(crate) fn stream_families(&self) -> &[crate::runtime::UiAllocationStreamFamily] {
        self.frame_identity.families()
    }
    pub(crate) fn invalidation_families(
        &self,
    ) -> impl ExactSizeIterator<Item = crate::runtime::UiAllocationInvalidationFamily> + '_ {
        self.frame_identity
            .invalidations()
            .iter()
            .map(crate::runtime::UiAllocationInvalidationIntent::family)
    }
    pub(crate) fn ingress_policy_verdicts(
        &self,
    ) -> &[crate::runtime::UiAllocationIngressPolicyVerdict] {
        self.frame_identity.ingress_policy_verdicts()
    }
    pub(crate) fn ordered_neighborhoods(
        &self,
    ) -> impl ExactSizeIterator<Item = &UiAllocationNeighborhoodIdentity> {
        self.ordered
            .iter()
            .map(UiAdmittedReplanNeighborhood::identity)
    }
    pub(crate) fn widen_reasons(
        &self,
    ) -> impl ExactSizeIterator<Item = Option<UiReplanWidenReason>> + '_ {
        self.ordered
            .iter()
            .map(UiAdmittedReplanNeighborhood::widen_reason)
    }
    pub(crate) fn expected_generations(
        &self,
    ) -> impl ExactSizeIterator<Item = crate::graph::UiReplanGenerationKey> + '_ {
        self.ordered
            .iter()
            .map(UiAdmittedReplanNeighborhood::generation_key)
    }
    pub(crate) fn frame_epoch(&self) -> crate::runtime::UiAllocationFrameEpoch {
        self.frame_identity.epoch()
    }
    pub(crate) fn policy(&self) -> crate::runtime::UiResolvedAllocationStreamPolicy {
        self.frame_identity.policy()
    }
    pub(crate) fn overlap_disposition(&self) -> UiReplanOverlapDisposition {
        self.overlap_disposition
    }
    pub(crate) fn root_posture(&self) -> UiReplanRootPosture {
        self.root_posture
    }
    pub(crate) fn consequences(&self) -> &super::UiGraphReplanConsequences {
        &self.consequences
    }
}

impl UiReplanNeighborhoodSelectionCounters {
    pub(in crate::graph::allocation_neighborhood) fn visit(
        &mut self,
    ) -> Result<(), UiReplanLocalityDenial> {
        increment(&mut self.invalidation_visits, 1)
    }
    pub(in crate::graph::allocation_neighborhood) fn prove(
        &mut self,
    ) -> Result<(), UiReplanLocalityDenial> {
        increment(&mut self.locality_proofs, 1)
    }
    pub(in crate::graph::allocation_neighborhood) fn consume_locality(
        &mut self,
        proof: &UiReplanLocalityProof,
    ) -> Result<(), UiReplanLocalityDenial> {
        increment(
            &mut self.graph_index_probes,
            proof.graph_membership_probes(),
        )?;
        increment(&mut self.replacement_consequence_reads, 1)
    }
    pub(in crate::graph::allocation_neighborhood) fn seal(
        &mut self,
        cardinality: usize,
    ) -> Result<(), UiReplanLocalityDenial> {
        self.set_cardinality =
            u16::try_from(cardinality).map_err(|_| UiReplanLocalityDenial::CounterExhausted)?;
        Ok(())
    }
    pub(in crate::graph::allocation_neighborhood) fn root_widen(
        &mut self,
    ) -> Result<(), UiReplanLocalityDenial> {
        increment(&mut self.root_widen_attempts, 1)
    }
    pub(in crate::graph::allocation_neighborhood) fn overlap_probe(
        &mut self,
        count: u16,
    ) -> Result<(), UiReplanLocalityDenial> {
        increment(&mut self.overlap_index_probes, count)
    }
    pub(in crate::graph::allocation_neighborhood) fn merged(
        &mut self,
    ) -> Result<(), UiReplanLocalityDenial> {
        increment(&mut self.merged_neighborhoods, 1)
    }
    pub fn invalidation_visits(self) -> u16 {
        self.invalidation_visits
    }
    pub fn locality_proofs(self) -> u16 {
        self.locality_proofs
    }
    pub fn set_cardinality(self) -> u16 {
        self.set_cardinality
    }
    pub fn root_widen_attempts(self) -> u16 {
        self.root_widen_attempts
    }
    pub fn graph_index_probes(self) -> u16 {
        self.graph_index_probes
    }
    pub fn replacement_consequence_reads(self) -> u16 {
        self.replacement_consequence_reads
    }
    pub fn overlap_index_probes(self) -> u16 {
        self.overlap_index_probes
    }
    pub fn merged_neighborhoods(self) -> u16 {
        self.merged_neighborhoods
    }
}

fn increment(target: &mut u16, count: u16) -> Result<(), UiReplanLocalityDenial> {
    *target = target
        .checked_add(count)
        .ok_or(UiReplanLocalityDenial::CounterExhausted)?;
    Ok(())
}
