use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::{
        fragment::RecordSelection, ApplicationEntityMarkerIdentity,
        ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
        ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalClockHandle, WorthQueryTemporalDueFieldWriter,
    WorthQueryTemporalOperationAuthorization, WorthQueryTemporalOperationInvoker,
    WorthQueryTemporalPrincipalSource, WorthQueryTemporalQueryAuthorization,
};
use worth_query_installation::facade::{
    WorthQueryAuthoredHostConditionalPredicateProvider, WorthQueryNamedClock,
    WorthQueryTemporalIntentProjector, WorthQueryTemporalIntentRevisionValue,
};

use super::{
    clock::{WorthQueryDesktopClockHandle, WorthQueryDesktopClockSource},
    installer::{
        install_desktop_temporal_operation, WorthQueryDesktopTemporalBindingSpec,
        WorthQueryDesktopTemporalInstallationDenial, WorthQueryInstalledDesktopTemporal,
    },
};

mod sealed {
    pub trait Sealed {}
}

/// Sealed declarative hook implemented exclusively by installer-paired desktop
/// temporal bindings so
/// callers of [`super::pump_temporal_catch_up_until`]
/// can resolve the runtime clock handle and caller-pumped desktop clock handle
/// with a guaranteed shared clock source state.
pub trait WorthQueryDesktopTemporalHomeBinding<Schema: ApplicationSchema>: sealed::Sealed {
    type Node: 'static;
    type Clock: WorthQueryNamedClock;

    fn clock_handle(&self) -> &WorthQueryConditionalClockHandle<Schema, Self::Node, Self::Clock>;
    fn desktop_clock(&self) -> &WorthQueryDesktopClockHandle<Self::Clock>;
}

/// Ready-to-use `Installed` type pairing [`WorthQueryInstalledDesktopTemporal`]
/// with the exact [`WorthQueryDesktopClockHandle`] derived from the installed
/// [`WorthQueryDesktopClockSource`].
///
/// Construct exclusively via [`install_desktop_temporal_clock_operation`] so
/// the runtime clock binding and caller-pumped clock handle are guaranteed to
/// share one underlying clock source state.
pub struct WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock> {
    temporal: WorthQueryInstalledDesktopTemporal<Schema, Node, Clock>,
    desktop_clock: WorthQueryDesktopClockHandle<Clock>,
}

impl<Schema, Node, Clock> sealed::Sealed
    for WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock>
{
}

impl<Schema, Node, Clock> WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock> {
    pub(super) const fn new(
        temporal: WorthQueryInstalledDesktopTemporal<Schema, Node, Clock>,
        desktop_clock: WorthQueryDesktopClockHandle<Clock>,
    ) -> Self {
        Self {
            temporal,
            desktop_clock,
        }
    }

    pub const fn temporal(&self) -> &WorthQueryInstalledDesktopTemporal<Schema, Node, Clock> {
        &self.temporal
    }

    pub const fn desktop_clock(&self) -> &WorthQueryDesktopClockHandle<Clock> {
        &self.desktop_clock
    }
}

impl<Schema, Node, Clock> WorthQueryDesktopTemporalHomeBinding<Schema>
    for WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock>
where
    Schema: ApplicationSchema,
    Node: 'static,
    Clock: WorthQueryNamedClock,
{
    type Node = Node;
    type Clock = Clock;

    fn clock_handle(&self) -> &WorthQueryConditionalClockHandle<Schema, Node, Clock> {
        self.temporal.clock_handle()
    }

    fn desktop_clock(&self) -> &WorthQueryDesktopClockHandle<Clock> {
        &self.desktop_clock
    }
}

/// Installs a desktop timed-intent operation backed by a
/// [`WorthQueryDesktopClockSource`] and derives the paired
/// [`WorthQueryDesktopClockHandle`] directly from that exact installed source.
#[allow(clippy::type_complexity)]
pub fn install_desktop_temporal_clock_operation<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
    Provider,
    Clock,
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
        WorthQueryDesktopClockSource<Clock>,
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
    WorthQueryInstalledDesktopTemporalClock<Schema, Node, Clock>,
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
    let desktop_clock = spec.clock_source.handle();
    let temporal = install_desktop_temporal_operation(installation, spec)?;
    Ok(WorthQueryInstalledDesktopTemporalClock::new(
        temporal,
        desktop_clock,
    ))
}
