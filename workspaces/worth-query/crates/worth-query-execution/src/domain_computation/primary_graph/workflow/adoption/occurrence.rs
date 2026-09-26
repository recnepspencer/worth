//! What program adoption found for each retained workflow definition and live
//! instance, and which dispositions the law leaves open for each.

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::{EntityId, RelationId};

/// Why a retained workflow definition cannot run under the target program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowIncompatibility {
    /// The target installs no vocabulary for the definition's spec.
    VocabularyUnsupported { spec: String },
    /// The target vocabulary no longer executes this node exactly as retained.
    NodeUncovered { node_path: String },
    /// A program name this node acts through changes meaning in the target.
    DependencyChanged { node_path: String, name: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowCompatibility {
    Compatible,
    Incompatible(WorthQueryWorkflowIncompatibility),
}

impl WorthQueryWorkflowCompatibility {
    pub const fn is_compatible(&self) -> bool {
        matches!(self, Self::Compatible)
    }
}

/// Effect custody one live instance holds under the source program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowInstanceCustody {
    /// No operation this instance ran has settled a performed effect.
    Unperformed,
    /// At least one operation settled under a performed-effect receipt.
    Performed,
    /// An approval authorizes an operation that has not settled yet, so its
    /// delivery or performed-effect recovery is still pending under the
    /// source program.
    ApprovalOutstanding { approval_node_path: String },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryWorkflowDefinitionDisposition {
    /// Keep the definition current; later starts run under the target.
    Carry,
    /// Stop new starts. Instances already pinned to it keep their own choice.
    Retire,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryWorkflowInstanceDisposition {
    /// Continue the instance under the target program.
    Carry,
    /// End the instance without running any further node.
    Cancel,
}

/// One current definition on the adopting branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowDefinitionOccurrence {
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) current_relation: RelationId,
    pub(super) spec: String,
    pub(super) compatibility: WorthQueryWorkflowCompatibility,
}

impl WorthQueryWorkflowDefinitionOccurrence {
    pub const fn lineage(&self) -> EntityId {
        self.lineage
    }

    pub const fn entity_id(&self) -> EntityId {
        self.definition
    }

    pub fn spec(&self) -> &str {
        &self.spec
    }

    pub const fn compatibility(&self) -> &WorthQueryWorkflowCompatibility {
        &self.compatibility
    }

    pub fn legal_dispositions(&self) -> &'static [WorthQueryWorkflowDefinitionDisposition] {
        super::legality::definition_dispositions(&self.compatibility)
    }
}

/// One live instance this exact branch incarnation authored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowInstanceOccurrence {
    pub(super) instance: EntityId,
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) live_membership: RelationId,
    pub(super) compatibility: WorthQueryWorkflowCompatibility,
    pub(super) custody: WorthQueryWorkflowInstanceCustody,
}

impl WorthQueryWorkflowInstanceOccurrence {
    pub const fn entity_id(&self) -> EntityId {
        self.instance
    }

    pub const fn lineage(&self) -> EntityId {
        self.lineage
    }

    /// The definition this instance is pinned to, current or not.
    pub const fn definition(&self) -> EntityId {
        self.definition
    }

    pub const fn compatibility(&self) -> &WorthQueryWorkflowCompatibility {
        &self.compatibility
    }

    pub const fn custody(&self) -> &WorthQueryWorkflowInstanceCustody {
        &self.custody
    }

    pub fn legal_dispositions(&self) -> &'static [WorthQueryWorkflowInstanceDisposition] {
        super::legality::instance_dispositions(&self.compatibility, &self.custody)
    }

    /// No disposition is legal: the instance holds custody the target cannot
    /// honor. Settle or recover it under the source program, or migrate it
    /// explicitly, then take a fresh inventory.
    pub fn requires_migration(&self) -> bool {
        self.legal_dispositions().is_empty()
    }
}

/// Every workflow fact a program change must decide on one branch, bound to
/// the exact owner truth it was read from by its digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowAdoptionInventory {
    pub(super) source: ApplicationProgramRevision,
    pub(super) target: ApplicationProgramRevision,
    pub(super) definitions: Box<[WorthQueryWorkflowDefinitionOccurrence]>,
    pub(super) instances: Box<[WorthQueryWorkflowInstanceOccurrence]>,
    pub(super) digest: [u8; 32],
    pub(super) work_units: usize,
}

impl WorthQueryWorkflowAdoptionInventory {
    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub fn definitions(&self) -> &[WorthQueryWorkflowDefinitionOccurrence] {
        &self.definitions
    }

    pub fn instances(&self) -> &[WorthQueryWorkflowInstanceOccurrence] {
        &self.instances
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty() && self.instances.is_empty()
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub const fn work_units(&self) -> usize {
        self.work_units
    }

    pub fn definition(&self, entity: EntityId) -> Option<&WorthQueryWorkflowDefinitionOccurrence> {
        self.definitions
            .iter()
            .find(|occurrence| occurrence.definition == entity)
    }

    pub fn instance(&self, entity: EntityId) -> Option<&WorthQueryWorkflowInstanceOccurrence> {
        self.instances
            .iter()
            .find(|occurrence| occurrence.instance == entity)
    }
}
