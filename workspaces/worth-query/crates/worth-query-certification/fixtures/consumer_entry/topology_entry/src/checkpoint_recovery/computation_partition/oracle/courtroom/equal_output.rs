//! A changed producer input republishes the same Length on its output body.
//! Its dependent advances through source rebuilding and ordinary input cutoff.
use super::*;
use worth_query_decl::facade::application_operation::ApplicationMutationBinding;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PositiveLength};
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
};

mod counted_producer;
mod counted_source;
mod dependent_publication;
mod installation;
mod publication;
mod readiness;
mod source;
worth_query_decl::facade::worth_query_application! { Schema { owner: "worth.query.certification.equal-output", version: (1, 0), contributions: [installation::Contribution], } }
impl TopologySchemaBinding for Schema {}

const OUTPUT: &str = "anchor-a";
const DEPENDENT: &str = "anchor-isolated";
const PRODUCER: &str = "courtroom-equal-output-producer";
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "region-output",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "region-output",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];

struct Producer;
struct Provider;
struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = source::SuccessorReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "courtroom-successor-source-family";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &source::SuccessorReadResult) -> &'static str {
        "region-output"
    }
}
struct Demand(&'static str);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> source::SuccessorRead {
        source::SuccessorRead {
            body_key: self.0.to_owned(),
        }
    }
}
impl WorthQueryApplicationProducerBinding<Schema> for Producer {
    type Operation = publication::Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = PlanarAnchorOutput<Schema>;
    const IDENTITY: &'static str = PRODUCER;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}
impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
    const SEMANTIC_IDENTITY: &'static str = "courtroom-equal-output-provider";
    fn operation_input(&self, source: &source::SuccessorReadResult) -> publication::Input {
        publication::Input {
            scope_key: source.body_key.clone(),
            entries: PositiveLength::get(&source.y).to_string(),
        }
    }
    fn idempotency_key(&self, _: &source::SuccessorReadResult, identity: &[u8; 32]) -> u64 {
        planar_source_key(identity) ^ 0x0006_12e0
    }
    fn demand_resources(
        &self,
        _: &source::SuccessorReadResult,
    ) -> WorthQueryProducerDemandResources {
        planar_producer_resources()
    }
}

macro_rules! settle {
    ($demand:expr, $request:expr) => {{
        let before = primary_graph::query_read_kernel_entries_on_this_thread_for_test();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            $demand.advance(&$request).unwrap()
        else {
            panic!("one advance settles")
        };
        (
            settled.producer_contacts_in_this_demand(),
            primary_graph::query_read_kernel_entries_on_this_thread_for_test() - before,
        )
    }};
}
#[test]
fn changed_input_equal_output_rebuilds_the_dependent_source_without_contacting_it() {
    let _guard = checkpoint_recovery_test_guard();
    let app = installation::install();
    let (scope, principal) = installation::authentication_fixture::authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut root = request
        .demand(Demand(OUTPUT))
        .start_in_program::<installation::Program, installation::Root>(&app)
        .unwrap();
    let mut dependent = request
        .demand(counted_producer::Demand)
        .start_in_program::<installation::Program, installation::DependentRoot>(&app)
        .unwrap();
    assert_eq!(settle!(root, request).0, 1);
    let initial_contacts = settle!(dependent, request).0;
    assert_eq!(initial_contacts, 1);
    let before = request
        .query(PlanarOutputRead {
            body_key: OUTPUT.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    for value in [2, 1, 3] {
        let source = request
            .query(PlanarRead {
                body_key: "anchor-b".to_owned(),
            })
            .execute()
            .unwrap();
        let changed = request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-b".to_owned(),
                replacement_y: length(value),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&(value + 100))
            .execute_performed::<installation::Program, installation::Root>(
                &app,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            changed,
            worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ));
        let mut root = request
            .demand(Demand(OUTPUT))
            .start_in_program::<installation::Program, installation::Root>(&app)
            .unwrap();
        assert_eq!(
            settle!(root, request).0,
            1,
            "the ordinary producer executes once for the changed input"
        );
        let expected = value.div_ceil(2);
        let actual = request
            .query(PlanarOutputRead {
                body_key: OUTPUT.to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value;
        assert_eq!(PositiveLength::get(&actual), expected);
        if expected == 1 {
            assert_eq!(actual, before);
        }
        counted_source::take_calls();
        dependent_publication::take_calls();
        let settled = settle!(dependent, request);
        let expected_contacts = usize::from(expected != 1);
        assert_eq!(
            dependent_publication::take_calls(),
            expected_contacts,
            "only a changed consumed value contacts the dependent"
        );
        assert_eq!(settled.0, initial_contacts + expected_contacts);
        assert_eq!(
            counted_source::take_calls(),
            1,
            "the dependent rebuilds its source once"
        );
    }
    // The dependent keeps its last row while its upstream refreshes twice.
    for value in [4, 3] {
        let source = request
            .query(PlanarRead {
                body_key: "anchor-b".to_owned(),
            })
            .execute()
            .unwrap();
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-b".to_owned(),
                replacement_y: length(value),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&(value + 200))
            .execute_performed::<installation::Program, installation::Root>(
                &app,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let mut root = request
            .demand(Demand(OUTPUT))
            .start_in_program::<installation::Program, installation::Root>(&app)
            .unwrap();
        assert_eq!(settle!(root, request).0, 1);
    }
    counted_source::take_calls();
    dependent_publication::take_calls();
    assert_eq!(settle!(dependent, request).0, initial_contacts + 1);
    assert_eq!(
        counted_source::take_calls(),
        1,
        "two upstream refreshes need one dependent source rebuild"
    );
    assert_eq!(
        dependent_publication::take_calls(),
        0,
        "both refreshes preserved the consumed value"
    );
}

mod indexed_decision;
mod witness;
