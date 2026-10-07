//! Refusals while reconstructing a suspended generated output.

use super::super::WorthQuerySuspendedGeneratedOutput;

/// Why a reconstruction step was refused. The refused step changed nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputReconstructionDenial {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
    /// The suspended output belongs to another runtime.
    ForeignRuntime,
    /// The suspended output was produced by a different producer.
    ForeignProducer,
    /// The producer's provider changed, or the producer is no longer installed.
    StaleProducerVersion,
    /// The runtime no longer records this output for its source.
    StaleOutputLineage,
    /// The suspended output bound no entity to the exactly-one output role.
    MissingOutputRole,
    /// The role's cardinality differs from the one the producer's installed
    /// contract records for it.
    OutputRoleCardinalityMismatch,
    /// The entity is not in the suspension manifest.
    MissingEntity,
    /// The role is retired or its recorded action is not admitted by the declaration.
    OutputRoleActionMismatch,
    /// A preserved output identity is not live outside this suspension.
    MissingRetainedEntity,
    /// The entity type is undeclared or differs from the manifest.
    EntityKindMismatch,
    /// The entity was already claimed.
    DuplicateEntityClaim,
    /// The generated entity handle belongs to another reconstruction.
    ForeignEntityHandle,
    /// The retained entity type is undeclared or is not the relation's declared
    /// endpoint.
    RetainedEntityKindMismatch,
    /// The retained entity handle belongs to another reconstruction.
    ForeignRetainedEntityHandle,
    /// The field is not declared for the entity.
    UnknownField,
    /// The field was already set on this entity.
    DuplicateField,
    /// The value could not be encoded for the field.
    InvalidFieldValue,
    /// The relation is not declared.
    MissingRelation,
    /// The relation was already claimed.
    DuplicateRelationClaim,
    /// More than one relation in the manifest matches.
    AmbiguousRelation,
    /// No relation in the manifest connects these endpoints.
    WrongRelationEndpoint,
    /// Some entities or relations in the manifest were never claimed.
    IncompleteManifest,
}

/// A refused reconstruction, with the suspended output handed back unchanged so
/// it can be reconstructed again.
pub struct WorthQueryGeneratedOutputReconstructionFailure {
    denial: WorthQueryGeneratedOutputReconstructionDenial,
    suspended: WorthQuerySuspendedGeneratedOutput,
}

impl WorthQueryGeneratedOutputReconstructionFailure {
    pub const fn denial(&self) -> WorthQueryGeneratedOutputReconstructionDenial {
        self.denial
    }

    pub fn into_suspended(self) -> WorthQuerySuspendedGeneratedOutput {
        self.suspended
    }
}

pub(super) fn reconstruction_failure(
    suspended: WorthQuerySuspendedGeneratedOutput,
    denial: WorthQueryGeneratedOutputReconstructionDenial,
) -> WorthQueryGeneratedOutputReconstructionFailure {
    WorthQueryGeneratedOutputReconstructionFailure { denial, suspended }
}
