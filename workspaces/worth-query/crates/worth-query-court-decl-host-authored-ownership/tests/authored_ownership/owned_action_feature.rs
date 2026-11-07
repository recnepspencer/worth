//! A complete feature references its correction-owned action and installs one handler.
use super::{lifecycle::*, *};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
struct Delivery;
impl<S: Scenario> ApplicationActionDelivery<S, CreateBinding<S>> for Delivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "ownership.reference-completion.v1";
}
struct Handler {
    observed: Option<Arc<AtomicUsize>>,
}
impl<S: Scenario> OperationHandler<S, CreateBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Create,
        reader: &mut DecisionReader<'_, '_, '_, S, CreateBinding<S>>,
    ) -> HandlerResult<CreationTargets<S>, ()> {
        lifecycle_handlers::Handler.decide(input, reader)
    }
    fn build_candidate(
        &self,
        input: &Create,
        decision: CreationTargets<S>,
        writer: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> HandlerResult<(), ()> {
        if let Some(observed) = &self.observed {
            observed.fetch_add(1, Ordering::SeqCst);
        }
        lifecycle_handlers::Handler.build_candidate(input, decision, writer)
    }
}
worth_query_host::facade::worth_query_application_contribution! {
    feature Definition<S: Scenario> for program::Entry => Runtime {
        schema: |schema| { schema };
        provides(program::Output);
        action_ref create(CreateBinding<S>, Delivery, Handler) {};
    }
}
worth_query_host::facade::worth_query_application_contribution! {
    feature Duplicate<S: Scenario> for program::Entry => DuplicateRuntime {
        schema: |schema| { schema };
        action create(CreateBinding<S>, Delivery, Handler) {
            operation: correction::UndoCreate::<S>::source_definition()
                .aftermath(DeclaredApplicationAftermathContract::correction_binding::<correction::UndoCreate<S>>()).finish();
        };
    }
}
thread_local! { static OBSERVER: std::cell::RefCell<Option<Arc<AtomicUsize>>> = const { std::cell::RefCell::new(None) }; }
struct Observe;
impl Drop for Observe {
    fn drop(&mut self) {
        OBSERVER.with(|value| {
            value.replace(None);
        });
    }
}
pub(super) fn configure_create<S: Scenario>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let observed = OBSERVER.with(|value| value.borrow().clone());
    Definition::<S>::configure(Runtime::<S>::new(Handler { observed }), setup)
}
fn builder() -> ApplicationSchemaDeclarationBuilder<Model<0>> {
    ApplicationSchemaDeclarationBuilder::from_fragment(OwnershipRoot::<Model<0>, false>(
        PhantomData,
    ))
    .unwrap()
}
#[test]
fn an_action_reference_preserves_one_correction_owned_operation() {
    let schema = Definition::<Model<0>>::declare_members(builder())
        .build()
        .unwrap();
    let operation = CreateOp::reference::<Model<0>>().name();
    assert_eq!(schema.erased().members().iter().filter(|member| matches!(member, ApplicationSchemaMember::Operation { operation: name, .. } if name == operation)).count(), 1);
}
#[test]
fn redeclaring_a_correction_owned_action_remains_a_duplicate_member() {
    assert_eq!(
        Duplicate::<Model<0>>::declare_members(builder())
            .build()
            .unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}
struct Program;
impl ApplicationProgramDefinition<Model<0>> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<program::Graph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ownership.reference-action.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = Definition::<Model<0>>::feature_specs();
        features.push(
            <program::Program as ApplicationProgramDefinition<Model<0>>>::feature_specs()
                .pop()
                .unwrap(),
        );
        features
    }
}
#[test]
fn a_feature_runs_its_referenced_correction_owned_action_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    OBSERVER.with(|value| {
        value.replace(Some(Arc::clone(&calls)));
    });
    let _observer = Observe;
    let host = correction_courts::install_definition::<Model<0>, Program>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let result = request
        .advanced()
        .mutate(Create {
            create_co: false,
            co_anchors: vec![],
            scope: 1,
            id: 10,
            depth: 0,
        })
        .without_source()
        .idempotency(&"reference-action".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    request.query(query::Input { id: 10 }).execute().unwrap();
}
