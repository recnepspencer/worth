//! Ordinary delivery composes the reviewed complete membership guard with its original query.
use super::*;
use crate::application_invariant_acceptance::membership::{
    self, Fanout, FanoutRoot, MembershipProgram, MembershipSchema as S,
};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationDeliveryDenial as DeliveryDenial,
    WorthQueryOperationDeliveryPolicy as Policy,
};
use worth_query_topology_entry::{
    PlanarBodyOutput, PlanarDiscoveryLink, PlanarOutputRead, PlanarOutputReadBinding,
    PlanarSourceFeature,
};
type B = PlanarSourceAdjustmentBinding<S>;
struct Delivery;
impl ApplicationActionDelivery<S, B> for Delivery {
    type Mode = ApplicationDiscoveredDeliveryMode<
        FanoutRoot,
        ApplicationQueryProjection<PlanarOutputReadBinding<S>>,
    >;
    const IDENTITY: &'static str = "membership.ordinary.declared-output.v1";
}
impl ApplicationDiscoveredActionDelivery<S, B> for Delivery {
    type Root = FanoutRoot;
    type Projection = ApplicationQueryProjection<PlanarOutputReadBinding<S>>;
}
impl ApplicationDiscoveredQueryDelivery<S, B> for Delivery {
    type Read = PlanarOutputReadBinding<S>;
    fn read_input(input: &PlanarSourceAdjustment) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: input.scope_key.clone(),
        }
    }
}
type Action = ApplicationActionReference<S, Fanout, PlanarSourceFeature, B, Delivery>;
struct MembershipDelivery;
impl ApplicationActionDelivery<S, worth_query_topology_entry::PlanarDiscoveryLinkBinding<S>>
    for MembershipDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "membership.ordinary.link-completion.v1";
}
type MembershipAction = ApplicationActionReference<
    S,
    ApplicationRootComposition,
    PlanarSourceFeature,
    worth_query_topology_entry::PlanarDiscoveryLinkBinding<S>,
    MembershipDelivery,
>;
struct Program;
impl ApplicationProgramDefinition<S> for Program {
    type Contributions = <MembershipProgram as ApplicationProgramDefinition<S>>::Contributions;
    type Outputs = <MembershipProgram as ApplicationProgramDefinition<S>>::Outputs;
    type Rules = <MembershipProgram as ApplicationProgramDefinition<S>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("membership.ordinary-current-delivery.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = MembershipProgram::feature_specs();
        specs.retain(|spec| {
            !(spec.feature().composition_instance() == ApplicationRootComposition::PATH
                && spec.feature().identity()
                    == <PlanarSourceFeature as ApplicationFeature<S>>::IDENTITY)
        });
        specs.push(ApplicationFeatureSpec::root::<S,PlanarSourceFeature>().provides::<PlanarBodyOutput>().native_action::<B>().finish_action().native_action::<worth_query_topology_entry::PlanarEditBinding<S>>().finish_action().action::<worth_query_topology_entry::PlanarDiscoveryLinkBinding<S>, MembershipDelivery>().finish_action().finish());
        specs.retain(|spec| {
            !(spec.feature().composition_instance() == Fanout::PATH
                && spec.feature().identity()
                    == <PlanarSourceFeature as ApplicationFeature<S>>::IDENTITY)
        });
        specs.push(
            ApplicationFeatureSpec::at::<S, Fanout, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .action::<B, Delivery>()
                .finish_action()
                .finish(),
        );
        specs
    }
}
#[test]
fn discovered_ordinary_membership_refuses_current_but_keeps_original_history_and_cleanup() {
    let world = membership::install_program::<Program>();
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "sibling-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let mut operation = Some(
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: "sibling-a".to_owned(),
                replacement_y: length(22),
            })
            .expect_source(source)
            .idempotency_owned(110_041)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<Program, Action>(&world.application)
            .unwrap_or_else(|(_, d)| panic!("construction: {d:?}")),
    );
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("ordinary native traversal: {denial:?}")
            }
        })
        .unwrap();
    assert_eq!(
        settled.settlement().output_count(),
        0,
        "negative original discovery is authentic evidence"
    );
    let before = settled.read_current(&request).unwrap();
    let historical_basis = settled.settlement().observation().selected_commit().clone();
    let operation = request
        .mutate(PlanarDiscoveryLink {
            scope_key: "sibling-a".to_owned(),
            target_key: "remote-b".to_owned(),
        })
        .without_source()
        .idempotency_owned(110_042)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program, MembershipAction>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("real membership construction: {d:?}"));
    let mut operation = Some(operation);
    use worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance;
    let completed = (0..16)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            WorthQuerySourceOnlyAdvance::Progress(next) => {
                operation = Some(next);
                None
            }
            WorthQuerySourceOnlyAdvance::Completed(completed) => Some(completed),
            _ => panic!("genuine ordinary membership effect"),
        })
        .unwrap();
    completed
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("membership cleanup: {d:?}"));
    assert!(matches!(
        settled.read_current_with_policy(
            &request,
            Policy::default()
        ),
        Err(DeliveryDenial::Stale(denial)) if denial.kind()==worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert_eq!(
        settled.settlement().observation().selected_commit(),
        &historical_basis
    );
    assert_eq!(
        settled.read_historical(&request).unwrap().rows(),
        before.rows()
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("historical close: {d:?}"));
}
