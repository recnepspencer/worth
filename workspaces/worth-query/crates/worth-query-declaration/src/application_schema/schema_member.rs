use worth_foundational::facade::{AspectContractRevision, AspectIdentity, ScalarAspectType};

use crate::application_aftermath::PortableApplicationAftermathContract;
use crate::application_capability::ErasedApplicationCapabilityContract;
use crate::application_query::ErasedApplicationQueryDefinition;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::authorization_policy::ApplicationAuthorizationPath;
use super::ApplicationRelationIntegrity;
use super::WorthQueryExternalEffectCorrelationFamily;
use super::{ApplicationFieldPresence, ApplicationMutationPreconditionTarget};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationProgramTarget {
    Create {
        entity: String,
    },
    Delete {
        entity: String,
    },
    Write {
        entity: String,
        aspect: String,
        field: String,
    },
    Link {
        relation: String,
        from: String,
        to: String,
    },
    Unlink {
        relation: String,
        from: String,
        to: String,
    },
    Emit {
        effect: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationDecisionReadTarget {
    Entity {
        entity: String,
    },
    Field {
        entity: String,
        aspect: String,
        field: String,
    },
    Relation {
        relation: String,
        from: String,
        to: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationSchemaMember {
    ApplicationMutation {
        description: crate::application_operation::ApplicationMutationDescription,
    },
    ApplicationInvariant {
        invariant: String,
        major: u16,
        minor: u16,
        execution_point: super::ApplicationInvariantExecutionPoint,
        maximum_work_units: std::num::NonZeroU64,
        enforcement: super::ApplicationInvariantEnforcement,
        required_groups: Vec<super::ApplicationInvariantGroup>,
        read_closure: Vec<super::ApplicationInvariantScopeTarget>,
        applicability: Vec<super::ApplicationInvariantScopeTarget>,
        provider: String,
        cost_posture: super::ApplicationInvariantCostPosture,
    },
    Entity {
        entity: String,
    },
    Aspect {
        entity: String,
        aspect: String,
        identity: AspectIdentity,
        revision: AspectContractRevision,
    },
    Field {
        entity: String,
        aspect: String,
        field: String,
        presence: ApplicationFieldPresence,
        scalar_family: ScalarAspectType,
        value_type: String,
        unit: Option<String>,
        frame: Option<String>,
        writable: bool,
        equality_queryable: bool,
    },
    Relation {
        relation: String,
        from: String,
        to: String,
        integrity: ApplicationRelationIntegrity,
    },
    PrincipalBinding {
        binding: String,
        mapping_entity: String,
        identity_aspect: String,
        identity_field: String,
        status_aspect: String,
        status_field: String,
        target_relation: String,
        principal_entity: String,
        principal_identity_aspect: String,
        principal_identity_field: String,
        principal_identity_scalar_family: ScalarAspectType,
        principal_identity_value_type: String,
    },
    ApplicationQuery {
        definition: ErasedApplicationQueryDefinition,
    },
    ApplicationCapability {
        contract: ErasedApplicationCapabilityContract,
    },
    ApplicationCapabilityContext {
        context: String,
        context_type: WorthQueryPortableTypeIdentity,
    },
    ApplicationCapabilityContextEntitySlot {
        context: String,
        context_type: WorthQueryPortableTypeIdentity,
        slot: String,
        slot_type: WorthQueryPortableTypeIdentity,
        entity: String,
    },
    ApplicationCapabilityProvenance {
        provenance: String,
        provenance_type: WorthQueryPortableTypeIdentity,
    },
    Operation {
        operation: String,
        input_type: WorthQueryPortableTypeIdentity,
    },
    OperationProgram {
        operation: String,
        target: ApplicationOperationProgramTarget,
    },
    OperationDecisionRead {
        operation: String,
        target: ApplicationOperationDecisionReadTarget,
    },
    OperationMutationPrecondition {
        operation: String,
        target: ApplicationMutationPreconditionTarget,
    },
    OperationDecisionFactBudget {
        operation: String,
        maximum_fact_count: usize,
    },
    OperationProjectionWorkBudget {
        operation: String,
        maximum_work_units: usize,
    },
    /// The operation escapes the runtime into a named external correlation
    /// family. Absence of this member means the operation declares no external
    /// effect and pays nothing for one.
    OperationExternalEffect {
        operation: String,
        effect: String,
        rust_payload_type: WorthQueryPortableTypeIdentity,
        protocol: super::ApplicationExternalEffectProtocol,
        maximum_payload_bytes: u64,
        correlation_family: WorthQueryExternalEffectCorrelationFamily,
    },
    /// Declared aftermath contract for one mutation operation.
    ///
    /// Absence means the operation carries no aftermath. Installation compiles
    /// this member into the operation's installed contracts; callers never
    /// supply an installed aftermath identity.
    OperationAftermath {
        operation: String,
        contract: PortableApplicationAftermathContract,
    },
    Policy {
        policy: String,
    },
    Ability {
        ability: String,
        scope_entity: String,
    },
    OperationAbility {
        operation: String,
        ability: String,
        scope_entity: String,
    },
    AbilityPolicy {
        ability: String,
        scope_entity: String,
        policy: String,
        paths: Vec<ApplicationAuthorizationPath>,
    },
    Unit {
        unit: String,
    },
    Effect {
        effect: String,
        payload_type: WorthQueryPortableTypeIdentity,
    },
}
