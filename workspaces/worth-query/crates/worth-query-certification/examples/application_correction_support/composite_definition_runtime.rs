//! Install the actual clock-owned conditional definition used by correction.
use super::{
    authentication,
    composite_definition_adapters::{IntentProjector, Invoker, PrincipalSource},
    composite_definition_contract::{self as contract, ReadyNode},
    composite_definition_model::Tick,
    composite_definition_query,
    schema::*,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_query::ApplicationQueryParameterSet,
        application_schema::ErasedApplicationSchemaDeclaration,
    },
    domain,
    primary_graph::*,
};
pub struct Clock;
pub const TICK_OUTPUT: u64 = 99;

struct TickOutputVersion;
impl domain::WorthQueryHostConditionalOutputVersionProvider<ReadyNode> for TickOutputVersion {
    fn semantic_identity(&self) -> &'static str {
        "numeric-definition.tick-output.v1"
    }

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        Ok(domain::WorthQueryHostProviderHeapRetention::none())
    }

    fn output_version(&self, _: u64) -> Result<u64, domain::WorthQueryHostPredicateFailure> {
        Ok(TICK_OUTPUT)
    }
}
impl domain::WorthQueryNamedClock for Clock {
    const PORTABLE_IDENTITY: &'static str = "numeric-definition.clock.v1";
}
pub struct ClockSource(AtomicU64);
impl domain::WorthQueryNamedClockSource<Clock> for ClockSource {
    const SEMANTIC_IDENTITY: &'static str = "numeric-definition.clock-source.v1";
    fn source_identity(&self) -> domain::WorthQueryClockSourceIdentity {
        domain::WorthQueryClockSourceIdentity::declare("numeric-definition-clock").unwrap()
    }
    fn timeline_identity(&self) -> domain::WorthQueryClockTimelineIdentity {
        domain::WorthQueryClockTimelineIdentity::declare("numeric-definition-timeline").unwrap()
    }
    fn observe(
        &self,
    ) -> Result<domain::WorthQueryNamedClockReading<Clock>, domain::WorthQueryNamedClockFailure>
    {
        Ok(domain::WorthQueryNamedClockReading::new(
            self.0.fetch_add(1, Ordering::SeqCst) + 1,
            domain::WorthQueryClockCoordinate::from_nanoseconds(1_000),
        ))
    }
}
pub struct Predicate(pub u64);
impl domain::WorthQueryHostConditionalPredicateProvider<ReadyNode> for Predicate {
    const SEMANTIC_IDENTITY: &'static str = "numeric-definition.predicate.v1";
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
        observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        let observed_parameter = observation.dependency(0).and_then(|dependency| {
            let domain::WorthQueryConditionalObservedValue::Present(artifact) =
                dependency.current()
            else {
                return None;
            };
            match artifact.field(&domain::FieldKey::new("Primary").unwrap()) {
                Some(domain::AspectValue::UInt64(value)) => Some(*value),
                _ => None,
            }
        });
        // Both authored rows (5 and 23) satisfy the dependency condition. At
        // either row, provider 5 allows the wake and provider 23 rejects it.
        // Skipping either definition replacement therefore changes execution.
        Ok(
            if observed_parameter.is_some_and(|value| value < 100) && self.0 < 10 {
                domain::WorthQueryHostPredicateDecision::Satisfied
            } else {
                domain::WorthQueryHostPredicateDecision::Unsatisfied
            },
        )
    }
}
pub struct Conditional;
impl WorthQueryApplicationConditionalBinding<NumericSchema> for Conditional {
    type Configuration = ();
    type Installed = WorthQueryConditionalClockHandle<NumericSchema, ReadyNode, Clock>;
    type Operation = Tick;
    const IDENTITY: &'static str = "numeric-definition.conditional.v1";
    fn package_contract(
        _: &ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        Ok(WorthQueryApplicationConditionalPackageContract::new(
            contract::definition().into_portable(),
            contract::binding().portable().clone(),
            ReadyNode::reference().node_identity(),
        ))
    }
    fn install(
        _: (),
        _: &WorthQueryApplicationConditionalProducerAccess<'_, NumericSchema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<NumericSchema>,
    ) -> Result<Self::Installed, WorthQueryConditionalRuntimeInstallationDenial> {
        let schema = installation.installed_schema();
        let principal = schema.principal_binding(ActorBinding::reference()).unwrap();
        let authentication = authentication::admit_operator_adapter(schema);
        let operation = schema.installed_operation(Tick::reference()).unwrap();
        let query = schema
            .installed_query_binding::<composite_definition_query::Binding>()
            .unwrap();
        let conditional = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &contract::binding())
            .unwrap()
            .bind_node(ReadyNode::reference())
            .unwrap()
            .bind_host_predicate_provider(Predicate(5))
            .unwrap()
            .bind_host_output_version_provider(TickOutputVersion)
            .unwrap()
            .bind_named_clock::<Clock, _>(ClockSource(AtomicU64::new(0)))
            .unwrap()
            .bind_temporal_intent_projection(
                query.into_query(),
                ApplicationQueryParameterSet::new(),
                IntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 13, 8).unwrap(),
            )
            .unwrap();
        let invariant = Arc::new(installation.retain_invariant_projection_authority());
        let execution = WorthQueryTemporalOperationExecution::with_authorization(
            invariant,
            Invoker,
            Key::reference(),
            Primary::reference(),
            Ceiling::reference(),
            100_u64,
            101_u64,
            WorthQueryPublicTemporalOperationAuthorization,
        )
        .unwrap();
        let reconstruction = WorthQueryTemporalReconstructionAccess::new(
            principal,
            PrincipalSource(authentication),
            Key::reference(),
            "numeric".to_owned(),
        )
        .unwrap();
        installation.bind_temporal_operation(conditional, execution, reconstruction)
    }
}
