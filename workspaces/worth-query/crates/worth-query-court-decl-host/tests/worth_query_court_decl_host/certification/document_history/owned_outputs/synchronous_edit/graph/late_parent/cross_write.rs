//! Mutual cross-write (PR C C3b, plan §5 "mutual cross-write", B-budget):
//! readers P = `cross-reader(p)` and R = `cross-reader(r)` are healthy in one
//! batch; consumer C1 of P writes R's strict support `x:r`, consumer C2 of R
//! writes P's `x:p`. A converging pair reaches its fixed point and commits;
//! an oscillating pair stops with the producer round budget.
use super::super::super::interleaved::component_carriage::parent as owner;
use super::*;
use family::{cross_converge, cross_oscillate, cross_reader, decision_input};

/// `decision-input(a)` discovers both readers at once.
pub struct Readers;
impl ApplicationConnectionIdentity for Readers {
    const IDENTITY: &'static str = "owned-outputs.late-parent.cross-readers";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, cross_reader::Feature>
    for Readers
{
}
type ReadersConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    cross_reader::Feature,
    cross_reader::InputPort,
    Readers,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Readers {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = cross_reader::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![
            cross_reader::Demand("p".into()),
            cross_reader::Demand("r".into()),
        ])
    }
}

/// One cross-write program per writer family: each reader's consumer is its
/// writer, with the reader as its strict prerequisite supplier.
macro_rules! cross_program {
    ($module:ident, $family:ident, $tag:literal) => {
        pub mod $module {
            use super::*;
            pub struct Writer;
            impl ApplicationConnectionIdentity for Writer {
                const IDENTITY: &'static str =
                    concat!("owned-outputs.late-parent.cross-writer-", $tag);
            }
            impl
                ApplicationOccurrenceConnectionBinding<
                    Schema,
                    cross_reader::Feature,
                    $family::Feature,
                > for Writer
            {
            }
            type WriterConnection = ApplicationConnectionRef<
                Schema,
                cross_reader::Feature,
                cross_reader::OutputPort,
                $family::Feature,
                $family::InputPort,
                Writer,
            >;
            impl WorthQueryApplicationDependentOutputConnection<Schema> for Writer {
                type RootDemand = cross_reader::Demand;
                type Discovery = query::first_output::Read;
                type Demand = $family::Demand;
                const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
                fn discovery_from_root(
                    root: &cross_reader::Demand,
                ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
                    Ok(query::first_output::Read(format!("cross:{}", root.0)))
                }
                fn demands_from_discovery(
                    row: &query::Row,
                ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                    Ok(vec![$family::Demand(
                        row.key.strip_prefix("cross:").unwrap().into(),
                    )])
                }
            }
            impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Writer {
                type Prerequisites = owner::Read;
                type PrerequisiteRow = owner::ParentBinding;
                type Selection = owner::Selection;
                fn prerequisite_rows() -> Self::Selection {
                    owner::owner()
                }
                fn prerequisites_from_target(
                    target: &$family::Demand,
                ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
                    Ok(owner::Read(format!("cross:{}", target.0)))
                }
                fn parent_from_prerequisite(
                    row: &owner::Parent,
                ) -> Result<cross_reader::Demand, WorthQueryRequiredOutputConnectionDenial> {
                    Ok(cross_reader::Demand(row.key.clone()))
                }
            }
            pub struct Writers;
            impl ApplicationOccurrenceComponent<Schema> for Writers {
                type Connections = (WriterConnection,);
            }
            pub type Tree = ApplicationOutputGraph<
                program::Connection,
                ApplicationOutputEdge<ReadersConnection, ApplicationOutputOccurrences<Writers>>,
            >;
            pub struct Delivery;
            impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
                type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
                const IDENTITY: &'static str =
                    concat!("owned-outputs.late-parent.cross-delivery-", $tag);
            }
            impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
                type Root = Tree;
                type Projection = ApplicationNoProjection;
            }
            pub struct Program;
            impl ApplicationProgramDefinition<Schema> for Program {
                type Contributions = (Contribution,);
                type Outputs = ApplicationProgramOutputs<Tree>;
                type Rules = ApplicationRuleLeaf;
                const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
                    concat!("owned-outputs.late-parent.cross-program-", $tag),
                );
                fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                    let mut specs = super::super::super::hierarchy::Program::feature_specs();
                    specs[0] =
                        super::super::super::super::super::program::entry_spec::<Delivery>();
                    specs
                }
            }
        }
    };
}
cross_program!(converging, cross_converge, "converge");
cross_program!(oscillating, cross_oscillate, "oscillate");

use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// The default world plus models `p` and `r` (0) and the readers' members
/// `x:p` and `x:r` (1).
fn install<P>() -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let declaration = Schema::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, P>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (world::Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, graph, installed| {
            world::seed(graph, installed, 2, Some(7))?;
            for key in ["p", "r"] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Model::reference(),
                        WorthQueryApplicationEntityKey::new(key).unwrap(),
                    )
                    .field(ModelKey::at("ModelKey"), key.to_owned())
                    .field(ModelValue::at("ModelValue"), 0u64),
                )?;
                let member = format!("x:{key}");
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Member::reference(),
                        WorthQueryApplicationEntityKey::new(&member).unwrap(),
                    )
                    .field(MemberKey::at("MemberKey"), member.clone())
                    .field(MemberValue::at("MemberValue"), 1u64),
                )?;
            }
            Ok(())
        },
    )
    .unwrap()
}
/// One stage (`b = 7`, the seeded value) through `$tree`.
macro_rules! cross_edit {
    ($host:expr, $request:expr, $tree:ty) => {{
        let mut edit = $request
            .edit_envelope()
            .idempotency("late-parent-cross-write".to_owned())
            .runtime_stages($host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<$tree, _>(edit::Intent {
            key: "b".into(),
            value: 7,
        })
        .unwrap();
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    }};
}
fn first_output(request: &family::Request<'_, '_, '_>, key: &str) -> u64 {
    request
        .query(query::first_output::Read(key.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value
}

/// Writers write min(read + 1, 3). The fixed point of P = x:p = min(R + 1, 3)
/// and R = x:r = min(P + 1, 3) is P = R = 3 (P < 3 forces P = R + 1 and then
/// R = min(R + 2, 3) = 3, a contradiction), and each writer's own output is
/// the reader value it consumed: converge:p = converge:r = 3. The edit
/// commits exactly that world.
#[test]
fn a_converging_mutual_cross_write_commits_its_fixed_point() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install::<converging::Program>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let outcome: Outcome = cross_edit!(&host, request, converging::Tree);
    let events = _execution.wave_events_for_test();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}; {events:#?}");
    for key in ["cross:p", "cross:r", "converge:p", "converge:r"] {
        assert_eq!(first_output(&request, key), 3, "{key}; {events:#?}");
    }
}

/// Writers write read + 1: every round moves the other reader, so the pair
/// never converges. The edit stops deterministically with the route round
/// budget (B-budget: 64 starts per route) at reader `r`, after the two
/// readers and the two writers each started 64 times (256 producer
/// contacts), and publishes nothing.
#[test]
fn an_oscillating_mutual_cross_write_stops_at_the_round_budget() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install::<oscillating::Program>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome: Outcome = cross_edit!(&host, request, oscillating::Tree);
    let ran = host.producer_contacts_on_this_thread_for_test() - contacts;
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
        panic!("an oscillating cross-write must stop: {outcome:?}")
    };
    let detail = format!("{denial:?}");
    assert!(
        detail.contains("ProducerRouteRoundBudgetExceeded")
            && detail.contains("started: 64, limit: 64")
            && detail.contains("owned-outputs.cross-reader.initial"),
        "{detail}"
    );
    assert_eq!(ran, 256, "four routes, 64 starts each: {detail}");
    assert!(request
        .query(query::first_output::Read("oscillate:p".into()))
        .execute()
        .is_err());
}
