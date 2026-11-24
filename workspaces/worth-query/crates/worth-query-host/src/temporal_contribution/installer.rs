use std::sync::Arc;

use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryParameterSet, ApplicationQueryScopeBinding,
    },
    application_schema::{
        fragment::RecordSelection, ApplicationEntityMarkerIdentity,
        ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
        ApplicationOperationRef, ApplicationPrincipalBindingRef,
        ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationInvariantProjectionAuthority, WorthQueryApplicationProjection,
    WorthQueryConditionalApplicationRuntimeInstallation, WorthQueryConditionalClockHandle,
    WorthQueryConditionalRuntimeInstallationDenial, WorthQueryNoTemporalDueField,
    WorthQueryPublicTemporalOperationAuthorization, WorthQueryPublicTemporalQueryAuthorization,
    WorthQueryTemporalDueFieldWriter, WorthQueryTemporalOperationAuthorization,
    WorthQueryTemporalOperationExecution, WorthQueryTemporalOperationInvoker,
    WorthQueryTemporalPrincipalSource, WorthQueryTemporalQueryAuthorization,
    WorthQueryTemporalReconstructionAccess,
};
use worth_query_installation::facade::{
    WorthQueryApplicationConditionalOperationBinding,
    WorthQueryAuthoredHostConditionalPredicateProvider,
    WorthQueryConditionalApplicationOperationDenial, WorthQueryConditionalNodeRef,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentBounds,
    WorthQueryTemporalIntentProjector, WorthQueryTemporalIntentRevisionValue,
};

use super::{
    clock::WorthQueryDesktopClockHandle, home_binding::WorthQueryInstalledDesktopTemporalClock,
};

/// Refusal while assembling and installing a desktop temporal contribution.
#[derive(Debug)]
pub enum WorthQueryDesktopTemporalInstallationDenial {
    /// Required schema member (operation, principal binding, or query) was missing.
    SchemaMemberMissing(&'static str),
    /// Conditional node, clock, or temporal-intent projection contract refused.
    ConditionalOperation(WorthQueryConditionalApplicationOperationDenial),
    /// Execution or reconstruction identity contract refused.
    Contract(&'static str),
    /// Runtime installation refused the assembled temporal operation.
    RuntimeInstallation(WorthQueryConditionalRuntimeInstallationDenial),
}

impl std::fmt::Display for WorthQueryDesktopTemporalInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaMemberMissing(role) => {
                write!(formatter, "desktop temporal schema member missing: {role}")
            }
            Self::ConditionalOperation(denial) => write!(formatter, "{denial:?}"),
            Self::Contract(detail) => formatter.write_str(detail),
            Self::RuntimeInstallation(denial) => write!(formatter, "{denial:?}"),
        }
    }
}

impl std::error::Error for WorthQueryDesktopTemporalInstallationDenial {}

/// Installed desktop temporal handle returned by [`install_desktop_temporal_operation`].
///
/// Retains the exact runtime-issued [`WorthQueryConditionalClockHandle`] and the
/// installation's [`WorthQueryApplicationInvariantProjectionAuthority`].
pub struct WorthQueryInstalledDesktopTemporal<Schema, Node, Clock> {
    clock: WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    invariant: Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>>,
}

impl<Schema, Node, Clock> WorthQueryInstalledDesktopTemporal<Schema, Node, Clock> {
    pub const fn clock_handle(&self) -> &WorthQueryConditionalClockHandle<Schema, Node, Clock> {
        &self.clock
    }

    pub const fn invariant(
        &self,
    ) -> &Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>> {
        &self.invariant
    }

    /// Pairs this installed temporal handle with its caller-pumped
    /// [`WorthQueryDesktopClockHandle`] for single-call Home catch-up via
    /// [`crate::application_home::WorthQueryApplicationHome::catch_up_temporal`].
    pub const fn into_home_binding(
        self,
        desktop_clock: WorthQueryDesktopClockHandle<Clock>,
    ) -> WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock> {
        WorthQueryInstalledDesktopTemporalClock::new(self, desktop_clock)
    }
}

/// Complete caller-owned specification for installing one desktop timed-intent
/// operation through [`install_desktop_temporal_operation`].
pub struct WorthQueryDesktopTemporalBindingSpec<
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
    QueryBinding,
    Projector,
    PrincipalBinding,
    PrincipalMapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
    Scope,
    ScopeField,
    ScopeValue,
    PrincipalSource,
    Invoker,
    IntentEntity,
    IdentityField,
    RevisionField,
    LifecycleField,
    LifecycleValue,
    DueField = WorthQueryNoTemporalDueField,
    Authorization = WorthQueryPublicTemporalOperationAuthorization,
    QueryAuthorization = WorthQueryPublicTemporalQueryAuthorization,
> where
    Schema: ApplicationSchema,
    QueryBinding: ApplicationQueryBinding<Schema>,
    <QueryBinding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope:
        ApplicationEntityMarkerIdentity<Schema>,
{
    pub operation: ApplicationOperationRef<Schema, ApplicationOperation, Input>,
    pub conditional_binding: WorthQueryApplicationConditionalOperationBinding<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
    >,
    pub node: WorthQueryConditionalNodeRef<D, O, F, Node>,
    pub predicate: Provider,
    pub clock_source: Source,
    pub parameters: ApplicationQueryParameterSet<QueryBinding::Query>,
    pub projector: Projector,
    pub bounds: WorthQueryTemporalIntentBounds,
    pub principal_binding: ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
    pub principal_source: PrincipalSource,
    pub scope_field: ScopeField,
    pub scope_value: ScopeValue,
    pub invoker: Invoker,
    pub identity_field: IdentityField,
    pub revision_field: RevisionField,
    pub lifecycle_field: LifecycleField,
    pub active_lifecycle: LifecycleValue,
    pub completed_lifecycle: LifecycleValue,
    pub due_field: DueField,
    pub authorization: Authorization,
    pub query_authorization: QueryAuthorization,
    pub _marker: std::marker::PhantomData<fn() -> (Clock, QueryBinding, Scope, IntentEntity)>,
}

/// Assembles and binds a desktop timed-intent operation in one call through the
/// production [`WorthQueryConditionalApplicationRuntimeInstallation`] boundary.
///
/// Authoritative lifecycle, bounded projection, operation effects, recurring
/// rescheduling, and fresh principal admission remain strictly caller-owned.
#[allow(clippy::type_complexity)]
pub fn install_desktop_temporal_operation<
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
    QueryBinding,
    Projector,
    PrincipalBinding,
    PrincipalMapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
    Scope,
    ScopeField,
    ScopeValue,
    PrincipalSource,
    Invoker,
    IntentEntity,
    IdentityField,
    IdentityValue,
    RevisionField,
    RevisionValue,
    LifecycleField,
    LifecycleValue,
    DueField,
    Authorization,
    QueryAuthorization,
>(
    installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    spec: WorthQueryDesktopTemporalBindingSpec<
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
        QueryBinding,
        Projector,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeField,
        ScopeValue,
        PrincipalSource,
        Invoker,
        IntentEntity,
        IdentityField,
        RevisionField,
        LifecycleField,
        LifecycleValue,
        DueField,
        Authorization,
        QueryAuthorization,
    >,
) -> Result<
    WorthQueryInstalledDesktopTemporal<Schema, Node, Clock>,
    WorthQueryDesktopTemporalInstallationDenial,
>
where
    Schema: ApplicationSchema + 'static,
    ApplicationOperation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    ApplicationOperation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: Clone + Send + Sync + 'static,
    D: 'static,
    O: 'static,
    F: 'static,
    Node: 'static,
    Provider: WorthQueryAuthoredHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    QueryBinding: ApplicationQueryBinding<Schema> + 'static,
    QueryBinding::ScopeBinding: ApplicationQueryScopeBinding<Schema, Scope = Scope>,
    QueryBinding::Query: 'static,
    <QueryBinding::ParameterBinding as ApplicationStructuredValueBinding>::Value: 'static,
    <QueryBinding::ResultBinding as ApplicationStructuredValueBinding>::Value:
        WorthQueryApplicationProjection<Schema, QueryBinding::Query> + 'static,
    Scope: ApplicationEntityMarkerIdentity<Schema> + 'static,
    Projector: WorthQueryTemporalIntentProjector<
        Node,
        Clock,
        <QueryBinding::ResultBinding as ApplicationStructuredValueBinding>::Value,
        Input,
    >,
    PrincipalBinding: 'static,
    PrincipalMapping: 'static,
    Principal: 'static,
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding:
        ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity> + 'static,
    ScopeField: RecordSelection<Entity = Scope, Value = ScopeValue> + 'static,
    ScopeField::Binding: ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeValue: Clone + Send + Sync + 'static,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
    QueryAuthorization: WorthQueryTemporalQueryAuthorization<
            Schema,
            QueryBinding::Query,
            <QueryBinding::ParameterBinding as ApplicationStructuredValueBinding>::Value,
            <QueryBinding::ResultBinding as ApplicationStructuredValueBinding>::Value,
            Principal,
            PrincipalIdentity,
            Scope,
        > + 'static,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, ApplicationOperation, Input, Scope>,
    IntentEntity: 'static,
    IdentityField: RecordSelection<Entity = IntentEntity, Value = IdentityValue> + 'static,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    IdentityValue: Clone + Send + 'static,
    RevisionField: RecordSelection<Entity = IntentEntity, Value = RevisionValue> + 'static,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue>
        + WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone + Send + 'static,
    LifecycleField: RecordSelection<Entity = IntentEntity, Value = LifecycleValue> + 'static,
    LifecycleField::Binding: ApplicationReadableScalarValueBinding<Value = LifecycleValue>,
    LifecycleValue: Clone + Send + Sync + 'static,
    DueField: WorthQueryTemporalDueFieldWriter<Schema, IntentEntity>,
    Authorization: WorthQueryTemporalOperationAuthorization<Schema, ApplicationOperation, Input, Scope>
        + 'static,
{
    let schema = installation.installed_schema();
    let principal_binding = schema
        .principal_binding(spec.principal_binding)
        .map_err(|_| {
            WorthQueryDesktopTemporalInstallationDenial::SchemaMemberMissing("principal_binding")
        })?;
    let operation = schema.installed_operation(spec.operation).map_err(|_| {
        WorthQueryDesktopTemporalInstallationDenial::SchemaMemberMissing("operation")
    })?;
    let query = schema
        .installed_query_binding::<QueryBinding>()
        .map_err(|_| {
            WorthQueryDesktopTemporalInstallationDenial::SchemaMemberMissing("reconstruction_query")
        })?;
    let conditional = installation
        .installed_packages()
        .bind_conditional_application_operation(operation, &spec.conditional_binding)
        .map_err(WorthQueryDesktopTemporalInstallationDenial::ConditionalOperation)?
        .bind_node(spec.node)
        .map_err(WorthQueryDesktopTemporalInstallationDenial::ConditionalOperation)?
        .bind_authored_host_predicate_provider(spec.predicate)
        .map_err(WorthQueryDesktopTemporalInstallationDenial::ConditionalOperation)?
        .bind_named_clock::<Clock, _>(spec.clock_source)
        .map_err(WorthQueryDesktopTemporalInstallationDenial::ConditionalOperation)?
        .bind_temporal_intent_projection(
            query.into_query(),
            spec.parameters,
            spec.projector,
            spec.bounds,
        )
        .map_err(WorthQueryDesktopTemporalInstallationDenial::ConditionalOperation)?;
    let invariant = Arc::new(installation.retain_invariant_projection_authority());
    let execution = WorthQueryTemporalOperationExecution::with_authorization_and_due_field(
        Arc::clone(&invariant),
        spec.invoker,
        spec.identity_field,
        spec.revision_field,
        spec.lifecycle_field,
        spec.active_lifecycle,
        spec.completed_lifecycle,
        spec.authorization,
        spec.due_field,
    )
    .map_err(WorthQueryDesktopTemporalInstallationDenial::Contract)?;
    let reconstruction = WorthQueryTemporalReconstructionAccess::with_query_authorization(
        principal_binding,
        spec.principal_source,
        spec.scope_field,
        spec.scope_value,
        spec.query_authorization,
    )
    .map_err(WorthQueryDesktopTemporalInstallationDenial::Contract)?;
    let clock = installation
        .bind_temporal_operation(conditional, execution, reconstruction)
        .map_err(WorthQueryDesktopTemporalInstallationDenial::RuntimeInstallation)?;
    Ok(WorthQueryInstalledDesktopTemporal { clock, invariant })
}
