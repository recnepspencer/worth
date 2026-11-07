//! Observed identity bounds from original durable records, never live authority.
use super::SignalOwnerReadmissionDenial as D;
use crate::durability::{SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor};
use worth_foundational::facade::{
    DurableSignalConditionalOccurrence, DurableSignalOwnerIdentity, SignalBranchAdvanceIdentity,
    SignalDefinitionTransitionIdentity,
};

/// Required contribution from the original World opens and intents. Standalone
/// owners explicitly supply `empty`; Home must supply its recorded contribution.
/// These bounds do not prove that an effect occurred or admit any branch.
#[derive(Clone, Debug)]
pub struct SignalRecordedIdentityContribution {
    owner: DurableSignalOwnerIdentity,
    pub(super) branch: u64,
    pub(super) cell: u64,
    pub(super) snapshot: Option<u64>,
    pub(super) admission: u64,
    pub(super) conditional: u64,
    pub(super) definition: u64,
    pub(super) advance: u64,
    nodes: worth_execution::ExecutionOrderedMap<
        worth_foundational::facade::DurableSignalBranchIdentity,
        worth_execution::ExecutionOrderedMap<u32, u32>,
    >,
}
impl SignalRecordedIdentityContribution {
    pub fn empty(owner: DurableSignalOwnerIdentity) -> Self {
        Self {
            owner,
            branch: 0,
            cell: 0,
            snapshot: None,
            admission: 0,
            conditional: 0,
            definition: 0,
            advance: 0,
            nodes: Default::default(),
        }
    }
    pub const fn owner(&self) -> DurableSignalOwnerIdentity {
        self.owner
    }
    pub fn observe_descriptor(
        &mut self,
        descriptor: &SignalDurableBranchDescriptor,
    ) -> Result<(), D> {
        self.check_owner(descriptor.owner())?;
        self.branch = self.branch.max(descriptor.branch().ordinal());
        self.cell = self.cell.max(descriptor.cell().ordinal());
        self.admission = self.admission.max(descriptor.admission().ordinal());
        if let Some(snapshot) = descriptor.snapshot() {
            self.snapshot = self.snapshot.max(Some(snapshot.ordinal()));
        }
        self.observe_definition(descriptor.definition().identity())
    }
    pub fn observe_definition(
        &mut self,
        identity: SignalDefinitionTransitionIdentity,
    ) -> Result<(), D> {
        self.check_owner(identity.owner())?;
        self.definition = self.definition.max(identity.ordinal());
        Ok(())
    }
    pub fn observe_advance(&mut self, identity: SignalBranchAdvanceIdentity) -> Result<(), D> {
        self.check_owner(identity.owner())?;
        self.advance = self.advance.max(identity.ordinal());
        Ok(())
    }
    pub fn observe_conditional(
        &mut self,
        identity: DurableSignalConditionalOccurrence,
    ) -> Result<(), D> {
        self.check_owner(identity.owner())?;
        self.conditional = self.conditional.max(identity.ordinal());
        Ok(())
    }
    pub fn observe_node(
        &mut self,
        node: worth_foundational::facade::DurableSignalNodeIdentity,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), D> {
        self.check_owner(node.branch().owner())?;
        self.branch = self.branch.max(node.branch().ordinal());
        let mut generations = self.nodes.get(&node.branch()).cloned().unwrap_or_default();
        if generations
            .get(&node.index())
            .is_none_or(|generation| *generation < node.generation())
        {
            generations
                .insert_cloned_native_for_request(&node.index(), &node.generation(), *execution)
                .map_err(D::OrderedStorage)?;
            self.nodes
                .insert_cloned_native_for_request(&node.branch(), &generations, *execution)
                .map_err(D::OrderedStorage)?;
        }
        Ok(())
    }
    pub(crate) fn observe_graph(
        &mut self,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
        graph: &crate::data::graph::SignalGraph,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), D> {
        self.check_owner(branch.owner())?;
        for (index, generation) in graph.authored_node_issuance_bounds() {
            self.observe_node(
                worth_foundational::facade::DurableSignalNodeIdentity::from_recorded(
                    branch, index, generation,
                ),
                execution,
            )?;
        }
        Ok(())
    }
    pub(crate) fn next_branch(&self) -> Result<u64, D> {
        self.branch.checked_add(1).ok_or(D::IdentityExhausted)
    }
    pub(crate) fn issue_advance_and_admission(
        &mut self,
    ) -> Result<
        (
            worth_foundational::facade::SignalBranchAdvanceIdentity,
            worth_foundational::facade::DurableSignalAdmissionIdentity,
        ),
        D,
    > {
        let advance = self.advance.checked_add(1).ok_or(D::IdentityExhausted)?;
        let admission = self.admission.checked_add(1).ok_or(D::IdentityExhausted)?;
        self.advance = advance;
        self.admission = admission;
        Ok((
            worth_foundational::facade::SignalBranchAdvanceIdentity::from_recorded(
                self.owner, advance,
            ),
            worth_foundational::facade::DurableSignalAdmissionIdentity::from_recorded(
                self.owner, admission,
            ),
        ))
    }
    pub(crate) fn issue_definition(&mut self) -> Result<SignalDefinitionTransitionIdentity, D> {
        self.definition = self.definition.checked_add(1).ok_or(D::IdentityExhausted)?;
        Ok(SignalDefinitionTransitionIdentity::from_recorded(
            self.owner,
            self.definition,
        ))
    }
    pub(crate) fn node_issuance_root(
        &self,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
    ) -> worth_execution::ExecutionOrderedMap<u32, u32> {
        self.nodes.get(&branch).cloned().unwrap_or_default()
    }
    pub(crate) fn next_snapshot(&self) -> Result<u64, D> {
        self.snapshot.map_or(Ok(0), |ordinal| {
            ordinal.checked_add(1).ok_or(D::IdentityExhausted)
        })
    }
    pub(super) fn observe_transition(
        &mut self,
        artifact: &SignalDefinitionTransitionArtifact,
    ) -> Result<(), D> {
        self.observe_definition(artifact.identity())?;
        self.branch = self.branch.max(artifact.branch().ordinal());
        for occurrence in artifact.conditional_occurrences() {
            self.observe_conditional(occurrence.occurrence())?;
        }
        Ok(())
    }
    fn check_owner(&self, owner: DurableSignalOwnerIdentity) -> Result<(), D> {
        if self.owner != owner {
            return Err(D::ForeignOwner);
        }
        Ok(())
    }
    pub(crate) fn check_issuable(&self) -> Result<(), D> {
        for value in [
            self.branch,
            self.cell,
            self.snapshot.unwrap_or(0),
            self.admission,
            self.conditional,
            self.definition,
            self.advance,
        ] {
            value.checked_add(1).ok_or(D::IdentityExhausted)?;
        }
        Ok(())
    }
}
