//! A real attestation declares the selected endpoint as its certified Subject.
pub use super::{
    super::{
        actions::{DenialBinding, ResultBinding},
        schema::*,
    },
    declaration::SelectedParent,
};
pub use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
pub use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;

#[derive(Clone, serde::Serialize)]
pub struct Sign(pub String);
worth_query_structured_value_binding!(pub Input for Sign {identity:"worth.query.reference.sign-input.v1",capture_input:true});
worth_query_operation!(pub SignReference for Schema,input Input);
worth_query_operation_reads!(SignReference => [Row, Parent, SelectedParent]);
worth_query_operation_creates!(SignReference=>[Evidence]);
worth_query_operation_links!(SignReference=>[History,Current]);

pub fn row(input: &Sign) -> String {
    input.0.clone()
}
worth_query_mutation_binding!(pub SignBinding for Sign,schema Schema,
identity "worth.query.reference.sign-binding.v1",input Input,operation SignReference,result ResultBinding,
idempotency u64,identity "worth.query.reference.sign-request.v1",decision WorthQueryOperationBodyDecision<Schema,Row>,denial DenialBinding,
handler identity "worth.query.reference.sign-body.v1",program required,outputs NoApplicationMutationOutputs,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope RowIdentity, field RowIdentity::at("RowIdentity"),value row,
);
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let body = SignReference::reference()
        .body()
        .in_scope(Row::reference())
        .unwrap()
        .create_in_scope(Evidence::reference(), "attestation")
        .unwrap()
        .capture_from_path(
            ApplicationQueryRootPath::from(Row::reference()).forward(SelectedParent::reference()),
            ParentIdentity::at("ParentIdentity"),
            SignedIdentity::at("SignedIdentity"),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .signed_by::<SignBinding>(
            Actor::at("Actor"),
            Provenance::at("Provenance"),
            "path-verification",
        )
        .unwrap()
        .supersede_on_subject_change(
            History::reference(),
            Current::reference(),
            Consequences::reference(),
            Record::at("Record"),
            std::num::NonZeroUsize::new(8).unwrap(),
        )
        .unwrap()
        .finish()
        .unwrap();
    builder
        .operation(
            SignReference::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        )
        .operation_read_field(SignReference::reference(), RowIdentity::at("RowIdentity"))
        .operation_projection_work_budget(SignReference::reference(), 256)
        .application_mutation_binding::<SignBinding>()
}

pub use crate::facade::certification::native_graph_input_guards::schema::storage_model::{
    Actor, Consequence, ConsequenceFacts, Consequences, Current, Evidence, Facts, History,
    Provenance, Record, SignedIdentity,
};
