//! A genuine NAV grant selects the complete installed graph through Native.
use super::*;
use worth_query_decl::facade::application_capability::*;
mod grant;
mod transfer;
worth_query_capability!(pub Navigate in Schema);
worth_query_capability_context!(pub Context in Schema);
worth_query_capability_provenance!(pub Provenance in Schema);
worth_query_operation!(pub SelectSnapshot for Schema, input query::model::ReadInput);
worth_query_operation_reads!(SelectSnapshot => [ModelKey]);

pub struct Navigation;
impl ApplicationHistoryNavigationBinding<Schema, History> for Navigation {
    const IDENTITY: &'static str = "OwnedModelSnapshotNavigation";
    type Capability = Navigate;
    type Input = query::model::Read;
    type InputBinding = query::model::ReadInput;
    type Operation = SelectSnapshot;
    fn input(access: &query::model::Read) -> Self::Input {
        query::model::Read(access.0.clone())
    }
}
impl ApplicationCapabilityRequest<Schema, Navigate> for query::model::Read {
    type Scope = Model;
    type Context = Context;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Model, Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(ModelKey::reference(), grant::text(&self.0)),
            grant::text("navigate"),
            grant::text("history"),
            ApplicationCapabilityRequestContext::new(Context::reference()),
        ))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    grant::declare(schema)
        .capability_context(Context::reference())
        .capability_provenance(Provenance::reference())
        .operation(
            SelectSnapshot::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(SelectSnapshot::reference(), ModelKey::reference())
        .capability(grant::contract())
}
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    grant::seed(graph)
}

#[test]
fn owned_graph_checkout_and_redo_restore_members_with_original_identity() {
    exercise_graph_checkout(world::install());
}
#[test]
fn worker_only_owned_graph_checkout_and_redo_preserve_original_member_identity() {
    exercise_graph_checkout(world::install_worker_only());
}
fn exercise_graph_checkout(host: world::Host) {
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.current_step(&request).unwrap();
    type Action = ApplicationActionReference<
        Schema,
        ApplicationRootComposition,
        Entry,
        edit::member::Binding,
        Completion,
    >;
    let mut edit = document
        .edit(edit::member::Intent(edit::Edit {
            key: "direct".into(),
            value: 717,
        }))
        .idempotency_owned(&request, "before-owned-checkout".to_owned())
        .unwrap()
        .retain_in_program::<Program, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let current = document.current_step(&request).unwrap();
    assert_ne!(current.id(), before.id());
    let edited = document.selection(&request).unwrap();
    let branch = edited.physical_endpoint().product_branch();
    assert_eq!(world::member_value(&host, branch, "direct"), 717);
    let identity = world::member_identity(&host, branch, "direct");
    for (step, expected_value, key) in [
        (before.id(), 11, 0x5353_1717),
        (current.id(), 717, 0x5353_1719),
    ] {
        let prepare = document.navigate::<Navigation>();
        let prepare = if step == before.id() {
            prepare.checkout(step)
        } else {
            prepare.redo_to(step)
        };
        let mut navigation = prepare
            .expected_selection(document.selection(&request).unwrap())
            .idempotency(key)
            .prepare(&request)
            .unwrap();
        let mut selected = false;
        for _ in 0..16 {
            navigation = match navigation.advance(&request) {
                WorthQueryHistoryNavigationAdvance::Pending(next) => next,
                WorthQueryHistoryNavigationAdvance::Selected(completion) => {
                    assert_eq!(completion.selection().step().id(), step);
                    selected = true;
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => panic!("{cause:?}"),
            };
        }
        assert!(selected, "bounded graph checkout must select");
        let branch = document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        assert_eq!(world::member_value(&host, branch, "direct"), expected_value);
        assert_eq!(world::member_identity(&host, branch, "direct"), identity);
        assert_eq!(world::member_value(&host, branch, "nested"), 22);
        assert_eq!(world::member_value(&host, branch, "foreign"), 33);
        assert_eq!(world::member_value(&host, branch, "reference"), 44);
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn a_foreign_incoming_edge_refuses_before_effect_with_public_subject_attribution() {
    type Action = ApplicationActionReference<
        Schema,
        ApplicationRootComposition,
        Entry,
        ownership_change::Binding,
        Completion,
    >;
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = document.current_step(&request).unwrap();
    let mut edit = document
        .edit(ownership_change::Change {
            from: "a".into(),
            to: "a".into(),
            member: "loose".into(),
            preserve_previous: true,
        })
        .idempotency_owned(&request, "import-loose-member".to_owned())
        .unwrap()
        .retain_in_program::<Program, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let expected = document.selection(&request).unwrap();
    let branch = expected.physical_endpoint().product_branch();
    let refusal = document
        .navigate::<Navigation>()
        .checkout(original.id())
        .expected_selection(expected.clone())
        .idempotency(0x5353_1718)
        .prepare(&request)
        .unwrap_err();
    let WorthQueryHistoryNavigationAdvanceDenial::Navigation(
        WorthQueryHistoryNavigationDenial::OwnedGraph(cause),
    ) = refusal
    else {
        panic!("expected actual graph refusal: {refusal:?}")
    };
    let foreign = cause
        .foreign_incoming_relation()
        .expect("foreign-edge retirement refusal names its actual source");
    let endpoint = foreign.entity();
    let owner = foreign.owner();
    assert_eq!(
        (
            endpoint.partition_id.as_u32(),
            endpoint.local_slot_value(),
            endpoint.generation_value()
        ),
        world::member_identity(&host, branch, "loose")
    );
    assert_eq!(
        (
            owner.partition_id.as_u32(),
            owner.local_slot_value(),
            owner.generation_value()
        ),
        world::model_identity(&host, branch, "b")
    );
    assert_eq!(document.selection(&request).unwrap(), expected);
    assert_eq!(world::member_value(&host, branch, "loose"), 55);
    drop(cause);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
