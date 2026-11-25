mod derivation;

use worth_query_declaration::facade::application_schema::{
    fragment::RecordSelection, ApplicationOperationMarkerIdentity, ApplicationOperationRef,
    ApplicationStructuredValueBinding, ErasedApplicationSchemaDeclaration,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationConditionalPackageContract, WorthQueryOutputReadinessContractDenial,
};
use worth_query_installation::facade as domain;

fn contract_denial(detail: impl Into<String>) -> WorthQueryOutputReadinessContractDenial {
    WorthQueryOutputReadinessContractDenial::from_detail(detail)
}

/// Default bounded per-execution work envelope for desktop temporal operations.
pub const DEFAULT_DESKTOP_TEMPORAL_MAX_WORK: u64 = 10_000;

/// Default bounded per-execution memory envelope for desktop temporal operations.
pub const DEFAULT_DESKTOP_TEMPORAL_MAX_MEMORY_BYTES: u64 = 1_048_576;

/// Declarative specification of one desktop timed-intent conditional node.
///
/// Derives the complete [`domain::WorthQueryDomainOperationDefinition`],
/// [`domain::WorthQueryApplicationConditionalOperationBinding`], and
/// [`WorthQueryApplicationConditionalPackageContract`] from the application
/// schema without requiring callers to hand-assemble semantic closures,
/// canonical queries, resource envelopes, or support tables.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDesktopTemporalNodeSpec {
    pub(super) domain_operation_name: String,
    pub(super) domain_operation_version: u32,
    pub(super) node_identity: String,
    pub(super) intent_entity: String,
    pub(super) identity_aspect: String,
    pub(super) identity_field: String,
    pub(super) gate_aspect: String,
    pub(super) gate_field: String,
    pub(super) wake: domain::WorthQueryTemporalWake,
    pub(super) after_nanoseconds: u64,
    pub(super) lowering_family: String,
    pub(super) max_work: u64,
    pub(super) max_memory_bytes: u64,
}

impl WorthQueryDesktopTemporalNodeSpec {
    /// Creates a desktop temporal node specification with bounded default
    /// resource limits and a monotonic clock wake.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_operation_name: impl Into<String>,
        domain_operation_version: u32,
        node_identity: impl Into<String>,
        intent_entity: impl Into<String>,
        identity_aspect: impl Into<String>,
        identity_field: impl Into<String>,
        gate_aspect: impl Into<String>,
        gate_field: impl Into<String>,
        lowering_family: impl Into<String>,
    ) -> Self {
        Self {
            domain_operation_name: domain_operation_name.into(),
            domain_operation_version,
            node_identity: node_identity.into(),
            intent_entity: intent_entity.into(),
            identity_aspect: identity_aspect.into(),
            identity_field: identity_field.into(),
            gate_aspect: gate_aspect.into(),
            gate_field: gate_field.into(),
            wake: domain::WorthQueryTemporalWake::MonotonicClock,
            after_nanoseconds: 1,
            lowering_family: lowering_family.into(),
            max_work: DEFAULT_DESKTOP_TEMPORAL_MAX_WORK,
            max_memory_bytes: DEFAULT_DESKTOP_TEMPORAL_MAX_MEMORY_BYTES,
        }
    }

    /// Creates a desktop temporal node specification directly from typed schema
    /// field selections on the same intent entity, deriving exact entity,
    /// native aspect key, and field identifiers without stringly-typed names.
    pub fn from_fields<IntentEntity, IdentityField, GateField>(
        domain_operation_name: impl Into<String>,
        domain_operation_version: u32,
        node_identity: impl Into<String>,
        identity_field: IdentityField,
        gate_field: GateField,
        lowering_family: impl Into<String>,
    ) -> Self
    where
        IdentityField: RecordSelection<Entity = IntentEntity>,
        GateField: RecordSelection<Entity = IntentEntity>,
    {
        Self::new(
            domain_operation_name,
            domain_operation_version,
            node_identity,
            identity_field.entity(),
            identity_field.aspect(),
            identity_field.field(),
            gate_field.aspect(),
            gate_field.field(),
            lowering_family,
        )
    }

    /// Selects the temporal wake mode (`MonotonicClock` or `WallClock`).
    pub const fn with_wake(mut self, wake: domain::WorthQueryTemporalWake) -> Self {
        self.wake = wake;
        self
    }

    /// Sets the portable `AfterNanoseconds` threshold (must be non-zero).
    pub const fn with_after_nanoseconds(mut self, after_nanoseconds: u64) -> Self {
        self.after_nanoseconds = after_nanoseconds;
        self
    }

    /// Overrides the bounded resource envelope for the generated domain operation.
    pub const fn with_resource_bounds(mut self, max_work: u64, max_memory_bytes: u64) -> Self {
        self.max_work = max_work;
        self.max_memory_bytes = max_memory_bytes;
        self
    }

    /// Derives the complete portable domain operation definition for this node.
    pub fn operation_definition<D, O, F>(
        &self,
        declaration: &ErasedApplicationSchemaDeclaration,
    ) -> Result<
        domain::WorthQueryDomainOperationDefinition<D, O, F>,
        WorthQueryOutputReadinessContractDenial,
    > {
        let gate_contract = domain::declared_application_aspect(
            declaration,
            &self.intent_entity,
            &self.gate_aspect,
        )
        .map_err(|denial| contract_denial(format!("gate aspect not declared: {denial:?}")))?
        .contract()
        .clone();
        let identity_contract = domain::declared_application_aspect(
            declaration,
            &self.intent_entity,
            &self.identity_aspect,
        )
        .map_err(|denial| contract_denial(format!("identity aspect not declared: {denial:?}")))?
        .contract()
        .clone();
        let gate_mask = domain::AspectMask::new([domain::CanonicalFieldPath::single(
            domain::FieldKey::new(&self.gate_field)
                .ok_or_else(|| contract_denial("invalid gate field key"))?,
        )]);
        let identity_mask = domain::AspectMask::new([domain::CanonicalFieldPath::single(
            domain::FieldKey::new(&self.identity_field)
                .ok_or_else(|| contract_denial("invalid identity field key"))?,
        )]);
        let gate_projection = domain::WorthQueryOperationNativeProjectionContract::new(
            gate_contract.clone(),
            gate_mask.clone(),
        )
        .map_err(|error| contract_denial(format!("invalid gate projection: {error:?}")))?;
        let native_projection = domain::WorthQueryOperationNativeProjectionContract::new(
            identity_contract,
            identity_mask,
        )
        .map_err(|error| contract_denial(format!("invalid native projection: {error:?}")))?;
        let node = derivation::build_temporal_node(self, gate_contract, gate_mask)?;
        let canonical_query = derivation::build_canonical_query(self)?;
        let resources = derivation::build_resource_contract(self)?;
        Ok(domain::WorthQueryDomainOperationDefinition::new(
            domain::WorthQueryDomainOperationIdentity::new(
                &self.domain_operation_name,
                self.domain_operation_version,
            ),
            domain::WorthQueryDomainOperationSemanticClosure {
                parameters: domain::WorthQueryOperationParameterContract::NotRequired,
                native_projection,
                canonical_query,
                collection: domain::WorthQueryOperationCollectionContract::NotCollection,
                required_capabilities: Vec::new(),
                required_domains: Vec::new(),
                workflow: domain::WorthQueryOperationWorkflowContract::NotRequired,
                evidence: domain::WorthQueryDomainEvidenceContract::not_required(),
                conditional_nodes: vec![node],
                graph_reads: domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
                    roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
                        role: "primary".into(),
                        participation:
                            domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                        access: domain::WorthQueryOperationGraphAccess::Project,
                        semantic_reads: vec![gate_projection],
                    }],
                },
                decision_facts: domain::WorthQueryOperationDecisionFactContract::NotRequired,
                touches: domain::WorthQueryOperationTouchContract::NotRequired,
                effects: domain::WorthQueryOperationEffectContract::NotRequired,
                invariants: domain::WorthQueryOperationInvariantContract::NotRequired,
                invariant_execution: domain::WorthQueryInvariantExecutionContract::NotRequired,
                replay: domain::WorthQueryOperationReplayContract::ReExecutable,
                aftermath: None,
                lineage: domain::WorthQueryOperationLineageContract::NotRequired,
                promotion: domain::WorthQueryOperationPromotionContract::NotRequired,
                publication: domain::WorthQueryOperationPublicationContract::DerivedProjection {
                    projection_role: domain::WorthQueryOperationProjectionRole::new("intent")
                        .map_err(|error| contract_denial(format!("{error:?}")))?,
                },
                projection_consumption:
                    domain::WorthQueryOperationProjectionConsumptionContract::QueryReadAuthority,
                terminal: domain::WorthQueryOperationTerminalContract {
                    result_states: vec![domain::WorthQueryOperationResultState::Ready],
                    failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
                },
                cost: domain::WorthQueryOperationCostContract {
                    lookup: domain::WorthQueryOperationCostClass::Constant,
                    execution: domain::WorthQueryOperationCostClass::Constant,
                    result_width: domain::WorthQueryOperationCostClass::Constant,
                },
                resources,
                support: derivation::temporal_support_contract(),
                lowering: domain::WorthQueryOperationLoweringContract {
                    family: self.lowering_family.clone(),
                    deterministic: true,
                },
            },
        ))
    }

    /// Derives the typed conditional-operation binding for one application operation.
    pub fn conditional_binding<Schema, Op, Input, D, O, F>(
        &self,
        declaration: &ErasedApplicationSchemaDeclaration,
        operation: ApplicationOperationRef<Schema, Op, Input>,
    ) -> Result<
        domain::WorthQueryApplicationConditionalOperationBinding<Schema, Op, Input, D, O, F>,
        WorthQueryOutputReadinessContractDenial,
    >
    where
        Schema: domain::ApplicationSchema,
        Op: ApplicationOperationMarkerIdentity<Schema>,
        Op::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        let definition = self.operation_definition::<D, O, F>(declaration)?;
        Ok(
            domain::WorthQueryApplicationConditionalOperationBinding::declare(
                operation,
                definition.reference(),
            ),
        )
    }

    /// Derives the complete package contract expected by
    /// `WorthQueryApplicationConditionalBinding::package_contract`.
    pub fn package_contract<Schema, Op, Input, D, O, F, Node>(
        &self,
        declaration: &ErasedApplicationSchemaDeclaration,
        operation: ApplicationOperationRef<Schema, Op, Input>,
        node: domain::WorthQueryConditionalNodeRef<D, O, F, Node>,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    >
    where
        Schema: domain::ApplicationSchema,
        Op: ApplicationOperationMarkerIdentity<Schema>,
        Op::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        let definition = self.operation_definition::<D, O, F>(declaration)?;
        let binding = domain::WorthQueryApplicationConditionalOperationBinding::declare(
            operation,
            definition.reference(),
        );
        Ok(WorthQueryApplicationConditionalPackageContract::new(
            definition.into_portable(),
            binding.portable().clone(),
            node.node_identity(),
        ))
    }
}

/// Zero-heap host predicate provider for timed intents whose readiness is
/// governed solely by authoritative `active` lifecycle and due coordinate.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryAlwaysReadyTemporalPredicate;

impl<Node: 'static> domain::WorthQueryHostConditionalPredicateProvider<Node>
    for WorthQueryAlwaysReadyTemporalPredicate
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.desktop-always-ready-predicate.v1";

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        Ok(domain::WorthQueryHostProviderHeapRetention::none())
    }

    fn evaluate(
        &self,
        _observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        Ok(domain::WorthQueryHostPredicateDecision::Satisfied)
    }
}

impl<Node: 'static> domain::WorthQueryAuthoredHostConditionalPredicateProvider<Node>
    for WorthQueryAlwaysReadyTemporalPredicate
{
    fn authored_provider_description(&self) -> domain::BridgeAuthoredProviderDescription {
        domain::BridgeAuthoredProviderDescription::new(
            domain::BoundaryProtocolIdentity::new("worth.query.host.desktop-always-ready"),
            domain::BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }

    fn readmit_authored_provider(
        &self,
        source: &domain::BridgeAuthoredProviderDescription,
    ) -> Result<Self, domain::AuthoredProviderReconstructionDenial> {
        <Self as domain::WorthQueryAuthoredHostConditionalPredicateProvider<Node>>::authored_provider_description(self)
            .admit_source_version(source)?;
        if !source.payload().is_empty() {
            return Err(domain::AuthoredProviderReconstructionDenial::MalformedPayload);
        }
        Ok(Self)
    }
}
