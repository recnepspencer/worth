//! A genuine NAV grant reaches the single custody seam without candidate effects.
use super::*;
use worth_query_decl::facade::application_capability::*;
mod grant;
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
fn owned_graph_checkout_reaches_the_single_custody_refusal_without_changing_the_document() {
    let host = world::install();
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
    let expected = document.selection(&request).unwrap();
    let branch = expected.physical_endpoint().product_branch();
    let value = world::member_value(&host, branch, "direct");
    let refusal = document
        .navigate::<Navigation>()
        .checkout(before.id())
        .expected_selection(expected.clone())
        .idempotency(0x5353_1717)
        .prepare(&request)
        .unwrap_err();
    let WorthQueryHistoryNavigationAdvanceDenial::Navigation(
        WorthQueryHistoryNavigationDenial::RequiresCascadeCustody { model },
    ) = refusal
    else {
        panic!("expected the single custody seam: {refusal:?}")
    };
    assert_eq!(
        (
            model.partition_id.as_u32(),
            model.local_slot_value(),
            model.generation_value()
        ),
        world::model_identity(&host, branch, "a")
    );
    assert_eq!(document.selection(&request).unwrap(), expected);
    assert_eq!(document.current_step(&request).unwrap().id(), current.id());
    assert_eq!(world::member_value(&host, branch, "direct"), value);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn a_foreign_incoming_edge_refuses_before_the_custody_seam_with_public_subject_attribution() {
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
        .expect("foreign-edge retirement refusal precedes custody admission");
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
