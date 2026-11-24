//! A successor round that reaches a record its own chain deleted, through a
//! link served at the chain head (#277 steps 1 to 3). The record is covered,
//! so the round reads its presence where it reads its fields: at the chain
//! head, where it exists. The chain's delete does not by itself make the
//! served list a prior-effect read; a link the chain did not make, at a record
//! the chain wrote, does. The round drops the record again, and Native folds
//! that delete onto the chain's own.
use super::super::super::super::program;
use super::super::hierarchy;
use super::*;
/// `$root(a)` under the edit, and `$child(a)` discovered from `first:a`.
macro_rules! drop_program {
    ($root:ident, $($child:ident)::+, $slug:literal) => {
        use super::*;
        use family::c3b::$root as root;
        use family::$($child)::+ as child;
        pub struct Start;
        impl ApplicationConnectionIdentity for Start {
            const IDENTITY: &'static str = concat!("owned-outputs.late-parent.", $slug, "-start");
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, program::Entry, root::Feature> for Start {}
        pub type Connection = ApplicationConnectionRef<
            Schema,
            program::Entry,
            program::EntryOutput,
            root::Feature,
            root::InputPort,
            Start,
        >;
        impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
            type Source = edit::Binding;
            type Demand = root::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn demand_from_source(
                _: &edit::Intent,
            ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok(root::Demand("a".into()))
            }
        }
        impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
            fn demand_from_source(
                input: &edit::Intent,
            ) -> Result<root::Demand, WorthQueryRequiredOutputConnectionDenial> {
                <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
            }
        }
        pub struct Discover;
        impl ApplicationConnectionIdentity for Discover {
            const IDENTITY: &'static str = concat!("owned-outputs.late-parent.", $slug, "-discover");
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, root::Feature, child::Feature> for Discover {}
        pub type ChildConnection = ApplicationConnectionRef<
            Schema,
            root::Feature,
            root::OutputPort,
            child::Feature,
            child::InputPort,
            Discover,
        >;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
            type RootDemand = root::Demand;
            type Discovery = query::first_output::Read;
            type Demand = child::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn discovery_from_root(
                _: &Self::RootDemand,
            ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
                Ok(query::first_output::Read("first:a".into()))
            }
            fn demands_from_discovery(
                row: &query::Row,
            ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                assert_eq!(row.key.as_str(), "first:a");
                Ok(vec![child::Demand("a".into())])
            }
        }
        pub type Tree = ApplicationOutputGraph<
            Connection,
            ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
        >;
        pub struct Delivery;
        impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
            type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
            const IDENTITY: &'static str = concat!("owned-outputs.late-parent.", $slug, "-delivery");
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
            const IDENTITY: ApplicationProgramIdentity =
                ApplicationProgramIdentity::new(concat!("owned-outputs.late-parent.", $slug, "-program"));
            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                let mut specs = hierarchy::Program::feature_specs();
                specs[0] = program::entry_spec::<Delivery>();
                specs
            }
        }
    };
}
/// The child only writes member `m`.
mod writer {
    drop_program!(child_drop, decision_write, "own-drop");
}
/// The child writes member `m` and links a child of its own under `first:a`.
mod linker {
    drop_program!(child_drop, c3b::kid_linker, "own-drop-linker");
}
/// One host on `$program`, and `$edit`: an edit of model `a` to a value, with
/// the wave events it produced.
macro_rules! drop_world {
    ($program:ident, $execution:ident, $host:ident, $request:ident, $edit:ident) => {
        let $execution = WorthQueryExecutionObservation::install_on_this_thread();
        let $host = world::install_program_with_other_model::<$program::Program>(
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
            2,
            Some(7),
        );
        let scope = request_scope();
        let principal = authenticate_operator($host.installed_schema(), &scope);
        let $request = $host.request(&principal, &scope);
        let $edit = |idempotency: &str, value| {
            let mut edit = $request
                .edit_envelope()
                .idempotency(idempotency.to_owned())
                .runtime_stages(&$host)
                .ordered_authored()
                .unwrap();
            edit.push_stage::<$program::Tree, _>(edit::Intent {
                key: "a".into(),
                value,
            })
            .unwrap();
            let outcome = edit
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            (outcome, $execution.wave_events_for_test())
        };
    };
}

/// The first edit (`a = 8`) derives `first:a` with its child `kid:a`, and
/// commits: 8 + 9, one child. The second (`a = 9`) re-enters `child-drop(a)`
/// in its Preserve posture. The head round unlinks and deletes `kid:a`; its
/// child `decision-write(a)` sets m = 10, and `first:a` reopens as a successor
/// round of its own chain. That round reads `first:a`'s children through the
/// link served at the chain head, reaches `kid:a`, and reads its presence.
/// `kid:a` is a record the chain wrote, so its presence is served where its
/// fields are, at the chain head: it exists there. The chain's unlink and
/// delete are recorded and served, and nothing else is at `first:a`, so the
/// round did not read its chain's effect. It goes on to unlink and delete the
/// child again. Native proves both against the chain's own and folds them to
/// nothing, so the edit commits what one round alone would: 9 + 10, no child.
/// Red before step 1: untyped `MissingAuthoritativeFact`, the presence read at
/// the current image, where the chain's delete had already removed `kid:a`.
/// Before step 2: `PriorEffectRead`, the chain's delete counted against the
/// list it was served. Before step 3: `StructuralEffect`, Native had no shape
/// for the round's delete.
#[test]
fn a_successor_dropping_again_the_record_its_chain_deleted_commits() {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    drop_world!(writer, execution, host, request, edit);
    let (outcome, events) = edit("own-output-drop-first", 8);
    assert!(
        matches!(outcome, Outcome::Committed(_)),
        "the first edit commits first:a and kid:a: {outcome:?}; {events:#?}"
    );
    assert_eq!(first(&request), Some(8 + 9));
    assert_eq!(kid(&request, "kid:a"), Some(0));
    let head = courts::native_head(&host);
    let (outcome, events) = edit("own-output-drop", 9);
    assert!(
        matches!(outcome, Outcome::Committed(_)),
        "the second edit commits the folded chain: {outcome:?}; {events:#?}"
    );
    assert!(
        events.iter().any(|event| event == "reopen task=0 owed=0"),
        "the chain derived a successor round: {events:#?}"
    );
    assert_ne!(courts::native_head(&host), head);
    assert_eq!(first(&request), Some(9 + 10));
    assert_eq!(kid(&request, "kid:a"), None);
}

/// The same two edits, with `kid-linker(a)` as the child. In the second edit
/// it sets m = 10 and links a child of its own, `linked-kid:a`, under
/// `first:a`, behind the head round that deleted `kid:a`. The successor's list
/// of `first:a`'s children then holds the chain's own link, served at the
/// chain head, and a link the chain did not make. Native's record of the
/// chain does not speak for that link: it was not there at the chain head, it
/// is there now, and it is at a record the chain wrote. The round read past
/// its chain's delete, so the edit stops typed and nothing publishes.
#[test]
fn a_successor_whose_chain_deleted_refuses_typed_on_a_link_the_chain_did_not_make() {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    drop_world!(linker, execution, host, request, edit);
    let (outcome, events) = edit("own-output-drop-linker-first", 8);
    assert!(
        matches!(outcome, Outcome::Committed(_)),
        "the first edit commits first:a and kid:a: {outcome:?}; {events:#?}"
    );
    assert_eq!(first(&request), Some(8 + 9));
    assert_eq!(kid(&request, "kid:a"), Some(0));
    assert_eq!(kid(&request, family::c3b::kid_linker::LINKED_KID), None);
    let head = courts::native_head(&host);
    let (outcome, events) = edit("own-output-drop-linker", 9);
    let detail = format!("{outcome:?}");
    assert!(
        detail.contains(
            "CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }"
        ),
        "{detail}; {events:#?}"
    );
    assert!(
        events.iter().any(|event| event == "reopen task=0 owed=0"),
        "the refused round is the chain's successor: {events:#?}"
    );
    assert_eq!(courts::native_head(&host), head);
    assert_eq!(first(&request), Some(8 + 9));
    assert_eq!(kid(&request, "kid:a"), Some(0));
    assert_eq!(kid(&request, family::c3b::kid_linker::LINKED_KID), None);
}

/// The value of `first:a` as a public read sees it.
pub(super) fn first(request: &family::Request<'_, '_, '_>) -> Option<u64> {
    let result = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .ok()?;
    result.rows().first().map(|row| row.value)
}

/// The value of child `key` as a public read sees it, if the record exists.
fn kid(request: &family::Request<'_, '_, '_>, key: &str) -> Option<u64> {
    let result = request
        .query(query::child_output::Read(key.into()))
        .execute()
        .ok()?;
    result.rows().first().map(|row| row.value)
}
