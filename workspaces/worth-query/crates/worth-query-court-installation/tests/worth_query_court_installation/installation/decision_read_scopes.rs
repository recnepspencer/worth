//! Shared and inline authoring install the same exact entity authority.
use worth_query_declaration::facade::application_schema::{fragment, *};
use worth_query_installation::facade::*;

struct Schema<const SHARED: bool>;
struct Records;
struct Allowed;
struct Inline;
struct Outside;
struct Common;
struct Read;
struct Inspect;
worth_query_declaration::worth_query_structured_value_binding!(Input for () { identity: "worth.tests.shared-input" });
impl<const SHARED: bool> ApplicationOperationMarkerIdentity<Schema<SHARED>> for Read {
    type InputBinding = Input;
    const IDENTIFIER: &'static str = "Read";
}
impl<const SHARED: bool> ApplicationOperationMarkerIdentity<Schema<SHARED>> for Inspect {
    type InputBinding = Input;
    const IDENTIFIER: &'static str = "Inspect";
}
impl OperationReads<Common> for Allowed {}
impl OperationReads<Read> for Allowed {}
impl OperationReads<Read> for Inline {}
impl OperationReads<Read> for Common {}
impl OperationReads<Inspect> for Common {}
impl OperationReads<Inspect> for Allowed {}
macro_rules! entities { ($($entity:ident),*) => { $(impl fragment::Entity for $entity {
    const NAME: &'static str = stringify!($entity); type Aspects = (); type Relations = ();
})* }; }
entities!(Allowed, Inline, Outside);
impl fragment::Fragment for Records {
    const NAME: &'static str = "Records";
    type Bindings = fragment::NoBindings;
    type Members = (Allowed, Inline, Outside);
}
impl<const SHARED: bool> ApplicationSchema for Schema<SHARED> {
    const OWNER: &'static str = "worth.tests.shared";
    const NAME: &'static str = "Shared";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let operation = ApplicationOperationRef::<Self, Read, ()>::from_declaration();
        let mut builder = ApplicationSchemaDeclarationBuilder::from_fragment(Records)?;
        if SHARED {
            let definition = ApplicationDecisionReadScopeDefinition::<Self, Common>::new()
                .entity(fragment::entity(Records, Allowed));
            builder = builder.decision_read_scope(definition);
            let reference = builder.decision_read_scope_reference::<Common>()?;
            builder = builder.operation_decision_read_scope(operation, reference);
            let reference = builder.decision_read_scope_reference::<Common>()?;
            let inspect = ApplicationOperationRef::<Self, Inspect, ()>::from_declaration();
            builder = builder
                .operation_decision_read_scope(inspect, reference)
                .operation(
                    inspect
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                );
        } else {
            builder = builder.operation_read_entity(operation, fragment::entity(Records, Allowed));
        }
        builder
            .operation_read_entity(operation, fragment::entity(Records, Inline))
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .build()
    }
}

fn installed_entities<const SHARED: bool>() -> Vec<String> {
    let declaration = Schema::<SHARED>::declaration().unwrap();
    let index = super::aspect_native_identity::model::index(&declaration);
    let installed = index.bind_application_schema(declaration).unwrap();
    let operation = installed
        .installed_operation(
            ApplicationOperationRef::<Schema<SHARED>, Read, ()>::from_declaration(),
        )
        .unwrap();
    let mut actual = operation
        .contracts()
        .graph_reads()
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .map(|scope| match scope {
            WorthQueryOperationGraphReadScope::Entity(entity) => entity.semantic_key().to_owned(),
            other => panic!("unexpected read scope: {other:?}"),
        })
        .collect::<Vec<_>>();
    actual.sort();
    actual
}

#[test]
fn shared_and_inline_authoring_install_the_same_exact_union() {
    assert_eq!(installed_entities::<true>(), vec!["Allowed", "Inline"]);
    assert_eq!(installed_entities::<false>(), vec!["Allowed", "Inline"]);
}

#[test]
fn operation_references_retain_the_same_compiled_scope_leaf() {
    let declaration = Schema::<true>::declaration().unwrap();
    let index = super::aspect_native_identity::model::index(&declaration);
    let schema = index.bind_application_schema(declaration).unwrap();
    println!(
        "shared scope typed fixture width=1 references=2 aggregate_bound_schema_canonical_bytes={}",
        schema
            .installation_canonical_work()
            .canonical_encoded_bytes()
    );
    let read = schema
        .installed_operation(ApplicationOperationRef::<Schema<true>, Read, ()>::from_declaration())
        .unwrap();
    let inspect = schema
        .installed_operation(
            ApplicationOperationRef::<Schema<true>, Inspect, ()>::from_declaration(),
        )
        .unwrap();
    let first = read
        .contracts()
        .graph_reads()
        .roles()
        .iter()
        .find(|role| role.shared_scope_identity().is_some())
        .unwrap();
    assert!(inspect.contracts().graph_reads().roles().iter().any(|role| role.role() == "primary"),
        "shared-only reads preserve the primary graph participation used by touch and invariant contracts");
    let second = inspect
        .contracts()
        .graph_reads()
        .roles()
        .iter()
        .find(|role| role.shared_scope_identity().is_some())
        .unwrap();
    assert_eq!(
        first.shared_scope_identity(),
        second.shared_scope_identity()
    );
    assert!(!first.read_scopes().is_empty());
    assert!(
        std::ptr::eq(first.read_scopes().as_ptr(), second.read_scopes().as_ptr()),
        "references must retain the actual compiled leaf instead of cloning its grants"
    );
}
