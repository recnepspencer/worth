//! External and inbound Directory operations must declare their required durable capabilities
//! regardless of correction shape (`no_aftermath()` included).
mod fixture;
use super::*;
use fixture::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionSetup,
    },
    application_entry::{
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenialKind as MutationDenial,
    },
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        SignalConditionalEvaluationBudget, WorthQueryExternalDispatchRequest,
        WorthQueryExternalEffectTransport, WorthQueryExternalTransportOutcome,
        WorthQueryInvariantMutationTarget, WorthQueryOperationDurableCapability,
        WorthQueryPrimaryGraphInstallationDenial,
    },
    product::WorthQueryDurableStateAdmissionDenial,
    runtime::{ExecutionAllocationPolicy, WorthQueryApplicationQueryResourceProfile},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::{
    application_home::schema::{Account, HomeRecords},
    application_runtime_surface as identity, checkpoint_capture,
};

/// Intentionally omits `declared_durable_capabilities()` (relying on default `Some(&[])`).
struct OmittedNotifyHandler(Arc<AtomicUsize>);
impl OperationHandler<OmittedSchema, OmittedNotifyBinding> for OmittedNotifyHandler {
    fn decide(
        &self,
        _: &u64,
        reader: &mut DecisionReader<'_, '_, '_, OmittedSchema, OmittedNotifyBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<OmittedSchema, Account>, ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let result: Result<_, HandlerExecutionDenial> = (|| {
            let entity =
                reader.resolve_entity(Number::at("AccountNumber"), "account-001".to_owned())?;
            let _ = reader.field(&entity, Balance::at("Balance"))?;
            Ok(reader.mutation_target(&entity)?)
        })();
        result.into()
    }
    fn build_candidate(
        &self,
        value: &u64,
        target: WorthQueryInvariantMutationTarget<OmittedSchema, Account>,
        writer: &mut CandidateWriter<'_, OmittedSchema, OmittedNotifyBinding>,
    ) -> HandlerResult<u64, ()> {
        let result: Result<u64, HandlerExecutionDenial> = (|| {
            let entity = writer.projected_entity(&target)?;
            writer.write_field(&entity, Balance::at("Balance"), *value)?;
            writer.emit_external(OmittedNoticeEffect::reference(), *value)?;
            Ok(*value)
        })();
        result.into()
    }
}

/// Declares emission+dispatch but omits `InboundCompletion` on an inbound contract.
struct PartialInboundHandler(Arc<AtomicUsize>);
impl OperationHandler<OmittedSchema, PartialInboundBinding> for PartialInboundHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::ExternalDispatch,
        ])
    }
    fn decide(
        &self,
        _: &u64,
        _: &mut DecisionReader<'_, '_, '_, OmittedSchema, PartialInboundBinding>,
    ) -> HandlerResult<(), ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        value: &u64,
        _: (),
        writer: &mut CandidateWriter<'_, OmittedSchema, PartialInboundBinding>,
    ) -> HandlerResult<u64, ()> {
        writer
            .emit_external(OmittedNoticeEffect::reference(), *value)
            .map(|()| *value)
            .into()
    }
}

pub(self) struct OmittedSurface;
impl ApplicationSchemaContribution<OmittedSchema> for OmittedSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.host.omitted.surface.v1");
    type Members = (
        HomeRecords,
        OmittedIdentityBinding,
        OmittedNoticeEffect,
        OmittedNotifyOp,
        OmittedNotifyBinding,
        PartialInboundOp,
        PartialInboundBinding,
    );
}
impl WorthQueryApplicationContribution<OmittedSchema> for OmittedSurface {
    type Configuration = Arc<AtomicUsize>;
    fn configure(
        contacts: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, OmittedSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<OmittedNotifyBinding, _>(OmittedNotifyHandler(contacts.clone()))?;
        setup.handler::<PartialInboundBinding, _>(PartialInboundHandler(contacts))?;
        Ok(())
    }
}

struct OmittedProgram;
struct OmittedFeature;
struct NotifyDelivery;
struct InboundDelivery;
impl ApplicationActionDelivery<OmittedSchema, OmittedNotifyBinding> for NotifyDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.host.omitted.notify-delivery.v1";
}
impl ApplicationActionDelivery<OmittedSchema, PartialInboundBinding> for InboundDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.host.omitted.inbound-delivery.v1";
}
impl ApplicationFeature<OmittedSchema> for OmittedFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.host.omitted.feature.v1";
}
impl ApplicationProgramDefinition<OmittedSchema> for OmittedProgram {
    type Contributions = (OmittedSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.omitted.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<OmittedSchema, OmittedFeature>()
                .action::<OmittedNotifyBinding, NotifyDelivery>()
                .finish_action()
                .action::<PartialInboundBinding, InboundDelivery>()
                .finish_action()
                .finish(),
        ]
    }
}

struct CountingTransport(Arc<AtomicUsize>);
impl WorthQueryExternalEffectTransport for CountingTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::SeqCst);
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn external_and_inbound_no_aftermath_handlers_refuse_when_required_capabilities_are_omitted() {
    let location = dispatch::location();
    let handler_contacts = Arc::new(AtomicUsize::new(0));
    let transport_contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let declaration = OmittedSchema::declaration().unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<OmittedSchema, OmittedProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (handler_contacts.clone(),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        WorthQueryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| seed(graph, installed),
    );
    let home: WorthQueryApplicationHome<OmittedSchema, OmittedProgram> =
        with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| panic!("omitted-capability Home opens: {refused}"));
    home.primary_graph()
        .install_external_effect_transport(Arc::new(CountingTransport(transport_contacts.clone())))
        .unwrap();

    let selected_commit = || {
        let app = home.primary_graph();
        let branch = app.current_world();
        let (_, probe_scope) = pending_support::scope();
        app.with_application_advancement(&probe_scope, |phase| {
            app.on_branch(branch)
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap()
    };
    let outbox_count = || {
        let app = home.primary_graph();
        let (_, probe_scope) = pending_support::scope();
        app.with_application_advancement(&probe_scope, |phase| {
            app.observe_committed_dispatch_outbox_count(&phase).unwrap()
        })
        .unwrap()
    };
    let before_commit = selected_commit();
    assert_eq!(
        outbox_count(),
        0,
        "fresh Home starts with zero outbox records"
    );
    let (_, scope) = pending_support::scope();
    let principal = identity::inspection_authentication::authenticate(
        home.primary_graph().installed_schema(),
        &scope,
    );

    let notify_denied = home
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(OmittedNotifyIntent(42))
        .without_source()
        .idempotency(&0x901_u64)
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    assert!(
        matches!(
            notify_denied.kind(),
            MutationDenial::DurableStateAdmission(
                WorthQueryDurableStateAdmissionDenial::UndeclaredSupplierCapabilities
            )
        ),
        "omitted external capabilities must refuse fail-closed: {notify_denied:?}"
    );

    let inbound_denied = home
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(PartialInboundIntent(43))
        .without_source()
        .idempotency(&0x902_u64)
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    assert!(
        matches!(
            inbound_denied.kind(),
            MutationDenial::DurableStateAdmission(
                WorthQueryDurableStateAdmissionDenial::UndeclaredSupplierCapabilities
            )
        ),
        "missing InboundCompletion capability must refuse fail-closed: {inbound_denied:?}"
    );

    assert_eq!(
        handler_contacts.load(Ordering::SeqCst),
        0,
        "zero handler contacts"
    );
    assert_eq!(
        transport_contacts.load(Ordering::SeqCst),
        0,
        "zero transport contacts"
    );
    assert_eq!(
        selected_commit(),
        before_commit,
        "selected branch head commit must stay unchanged"
    );
    assert_eq!(
        outbox_count(),
        0,
        "owner-observed committed dispatch outbox must contain zero records"
    );

    drop((notify_denied, inbound_denied, principal));
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("Home closes cleanly: {refused:?}"))
        .reclaim();
}
