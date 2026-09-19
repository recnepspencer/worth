use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryHostConditionalPredicateProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentProjector,
};

use super::installation::{
    ConditionalClockLease, WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind, WorthQueryPendingConditionalOperation,
};
use super::lifecycle::WorthQueryInstalledTemporalOperation;
use super::publication::ConditionalRuntimeAffinity;

pub(super) struct PendingTemporalOperation<Binding, Reconstruction, Execution> {
    binding_identity: Arc<super::canonical_identity::WorthQueryTemporalBindingIdentity>,
    clock_lease: Arc<ConditionalClockLease>,
    binding: Binding,
    reconstruction: Reconstruction,
    execution: Execution,
}

impl<Binding, Reconstruction, Execution>
    PendingTemporalOperation<Binding, Reconstruction, Execution>
{
    pub(super) fn new(
        binding_identity: Arc<super::canonical_identity::WorthQueryTemporalBindingIdentity>,
        clock_lease: Arc<ConditionalClockLease>,
        binding: Binding,
        reconstruction: Reconstruction,
        execution: Execution,
    ) -> Self {
        Self {
            binding_identity,
            clock_lease,
            binding,
            reconstruction,
            execution,
        }
    }
}

impl<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
        QueryAuthorization,
        Invoker,
        IntentEntity,
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        IdentityUnit,
        RevisionAspect,
        RevisionField,
        RevisionValue,
        RevisionWrite,
        RevisionEquality,
        RevisionUnit,
        LifecycleAspect,
        LifecycleField,
        LifecycleValue,
        LifecycleWrite,
        LifecycleEquality,
        LifecycleUnit,
        Authorization,
    > WorthQueryPendingConditionalOperation<Schema>
    for PendingTemporalOperation<
        WorthQueryInstalledTemporalConditionalOperation<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
            Provider,
            Clock,
            Source,
            Query,
            Parameters,
            QueryResult,
            Scope,
            Projector,
        >,
        super::reconstruction_authority::WorthQueryTemporalReconstructionAccess<
            Schema,
            PrincipalBinding,
            PrincipalMapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
            Scope,
            ScopeAspect,
            ScopeField,
            ScopeValue,
            ScopeWrite,
            ScopeUnit,
            PrincipalSource,
            QueryAuthorization,
        >,
        super::operation_invocation::WorthQueryTemporalOperationExecution<
            Schema,
            ApplicationOperation,
            Input,
            Scope,
            Invoker,
            IntentEntity,
            IdentityAspect,
            IdentityField,
            IdentityValue,
            IdentityWrite,
            IdentityUnit,
            RevisionAspect,
            RevisionField,
            RevisionValue,
            RevisionWrite,
            RevisionEquality,
            RevisionUnit,
            LifecycleAspect,
            LifecycleField,
            LifecycleValue,
            LifecycleWrite,
            LifecycleEquality,
            LifecycleUnit,
            Authorization,
        >,
    >
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
    ApplicationOperation: 'static,
    Input: Clone + Send + Sync + 'static,
    D: 'static,
    O: 'static,
    F: 'static,
    Node: 'static,
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Query: 'static,
    Parameters: 'static,
    QueryResult: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<Schema, Query>
        + 'static,
    Scope: 'static,
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
    PrincipalBinding: 'static,
    PrincipalMapping: 'static,
    Principal: 'static,
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding:
        worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
                Value = PrincipalIdentity,
            > + 'static,
    ScopeAspect: 'static,
    ScopeField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = ScopeValue>
        + 'static,
    ScopeField::Binding:
        worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeValue: Clone + Send + Sync + 'static,
    ScopeWrite: worth_query_installation::facade::WritePosture + 'static,
    ScopeUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    PrincipalSource: super::reconstruction_authority::WorthQueryTemporalPrincipalSource<Schema>,
    QueryAuthorization: super::WorthQueryTemporalQueryAuthorization<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        > + 'static,
    Invoker: super::operation_invocation::WorthQueryTemporalOperationInvoker<
        Schema,
        ApplicationOperation,
        Input,
        Scope,
    >,
    IntentEntity: 'static,
    IdentityAspect: 'static,
    IdentityField: worth_query_installation::facade::OperationReads<ApplicationOperation>
        + worth_query_installation::facade::DeclaredApplicationFieldValue<Value = IdentityValue>
        + 'static,
    IdentityField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
        Value = IdentityValue,
    >,
    IdentityValue: Clone + Send + 'static,
    IdentityWrite: worth_query_installation::facade::WritePosture + 'static,
    IdentityUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    RevisionAspect: 'static,
    RevisionField: worth_query_installation::facade::OperationReads<ApplicationOperation>
        + worth_query_installation::facade::OperationWrites<ApplicationOperation>
        + worth_query_installation::facade::DeclaredApplicationFieldValue<Value = RevisionValue>
        + 'static,
    RevisionField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = RevisionValue,
        > + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone + Send + 'static,
    RevisionWrite: worth_query_installation::facade::WritableCapability + 'static,
    RevisionEquality: 'static,
    RevisionUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    LifecycleAspect: 'static,
    LifecycleField: worth_query_installation::facade::OperationReads<ApplicationOperation>
        + worth_query_installation::facade::OperationWrites<ApplicationOperation>
        + worth_query_installation::facade::DeclaredApplicationFieldValue<Value = LifecycleValue>
        + 'static,
    LifecycleField::Binding:
        worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = LifecycleValue,
        >,
    LifecycleValue: Clone + Send + Sync + 'static,
    LifecycleWrite: worth_query_installation::facade::WritableCapability + 'static,
    LifecycleEquality: 'static,
    LifecycleUnit: worth_query_installation::facade::ApplicationFieldUnit + 'static,
    Authorization: super::WorthQueryTemporalOperationAuthorization<Schema, ApplicationOperation, Input, Scope>
        + 'static,
{
    fn binding_identity(&self) -> &str {
        self.binding_identity.support_identity()
    }

    fn install(
        self: Box<Self>,
        bridge: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        affinity: &ConditionalRuntimeAffinity,
        authoritative_commit_cursor: u64,
    ) -> Result<
        Box<dyn super::lifecycle::WorthQueryInstalledConditionalOperation<Schema>>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let request = super::predicate_admission::prepare_temporal_predicate_installation(
            &self.binding,
            graph,
        )?;
        let lowering = bridge
            .install_owned_conditional(request)
            .map_err(|denial| {
                WorthQueryConditionalRuntimeInstallationDenial::new(
                    WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                    format!("{:?}: {}", denial.kind(), denial.detail()),
                )
            })?;
        let runtime_canonical_identity =
            Arc::new(affinity.bind(&self.binding_identity).map_err(|denial| {
                WorthQueryConditionalRuntimeInstallationDenial::new(
                    WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                    format!("conditional runtime identity was denied: {denial:?}"),
                )
            })?);
        let runtime_binding_identity = Arc::clone(runtime_canonical_identity.bridge_identity());
        let installation_canonical_work = self
            .binding_identity
            .canonical_work()
            .combine(runtime_canonical_identity.canonical_work());
        Ok(Box::new(WorthQueryInstalledTemporalOperation {
            lifecycle_token: Default::default(),
            definition: Arc::new(super::definition::WorthQueryTemporalOperationDefinition {
                binding_identity: self.binding_identity,
                installation_canonical_work,
                clock_lease: self.clock_lease,
                binding: self.binding,
                reconstruction: self.reconstruction,
                execution: self.execution,
            }),
            bootstrap_lowering: lowering,
            active_affinity: None,
            managed_clock: None,
            runtime_binding_identity,
            runtime_canonical_identity,
            runtime_capability_identity: affinity.runtime_authority(),
            retained_wakes: Vec::new(),
            reconstructed_intents: std::collections::BTreeMap::new(),
            reconstruction_work: Default::default(),
            authoritative_commit_cursor,
            bootstrap_commit_catch_up_pending: true,
            commit_watch: Default::default(),
            operation_totals: Default::default(),
            pending_direct_delivery: None,
            inactive_bindings: Default::default(),
            next_evaluation_binding_ordinal: 1,
        }))
    }
}

pub(super) fn temporal_binding_identity<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
    Provider,
    Clock,
    Source,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Projector,
>(
    binding: &WorthQueryInstalledTemporalConditionalOperation<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >,
    principal_source_identity: &str,
    invoker_identity: &str,
) -> Result<
    Arc<super::canonical_identity::WorthQueryTemporalBindingIdentity>,
    worth_foundational::facade::CanonicalDigestDerivationDenial,
>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
{
    let node = binding.clocked_node().provider().node();
    super::canonical_identity::prepare_temporal_binding_identity(
        super::canonical_identity::TemporalBindingIdentityParts {
            node_authority: node.authority_identity(),
            clock: binding.clocked_node().clock_identity(),
            source: binding.clocked_node().source_identity().as_str(),
            timeline: binding.clocked_node().timeline_identity().as_str(),
            query: *binding.query().identity().digest(),
            projector: binding.projector_semantic_identity(),
            principal_source: principal_source_identity,
            invoker: invoker_identity,
        },
    )
    .map(Arc::new)
}
