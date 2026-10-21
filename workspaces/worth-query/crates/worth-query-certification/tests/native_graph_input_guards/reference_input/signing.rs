//! A real attestation declares the selected endpoint as its certified Subject.
use super::{
    super::{
        actions::{DenialBinding, ResultBinding},
        schema::*,
    },
    declaration::SelectedParent,
};
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_mutation_binding, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_relation, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;
worth_query_entity!(pub Evidence for Schema);
worth_query_entity!(pub Consequence for Schema);
worth_query_aspect!(pub Facts for Schema,Evidence;identity=AspectIdentity(0x9175D040),revision=AspectContractRevision(1),);
worth_query_aspect!(pub ConsequenceFacts for Schema,Consequence;identity=AspectIdentity(0x9175D041),revision=AspectContractRevision(1),);
worth_query_field!(pub SignedIdentity for Schema,Evidence,Facts:String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub Actor for Schema,Evidence,Facts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub Provenance for Schema,Evidence,Facts:ApplicationSigningGraphSubjectProvenance=>ApplicationSigningGraphSubjectProvenanceBinding,read_only,equality);
worth_query_field!(pub Record for Schema,Consequence,ConsequenceFacts:ApplicationSigningSupersession=>ApplicationSigningSupersessionBinding,read_only,equality);
worth_query_relation!(pub History in Schema,Row=>Evidence;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub Current in Schema,Row=>Evidence;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub Consequences in Schema,Evidence=>Consequence;integrity=same_context_unbounded_retain_dangling);
#[derive(Clone, serde::Serialize)]
pub struct Sign(pub String);
worth_query_structured_value_binding!(pub Input for Sign {identity:"worth.query.reference.sign-input.v1",capture_input:true});
worth_query_operation!(pub SignReference for Schema,input Input);
worth_query_operation_reads!(SignReference=>[Row,RowIdentity,Parent,ParentIdentity,SelectedParent]);
worth_query_operation_creates!(SignReference=>[Evidence]);
worth_query_operation_links!(SignReference=>[History,Current]);
worth_query_operation_writes!(SignReference=>[SignedIdentity,Actor,Provenance]);
fn row(input: &Sign) -> String {
    input.0.clone()
}
worth_query_mutation_binding!(pub SignBinding for Sign,schema Schema,
identity "worth.query.reference.sign-binding.v1",input Input,operation SignReference,result ResultBinding,
idempotency u64,identity "worth.query.reference.sign-request.v1",decision WorthQueryOperationBodyDecision<Schema,Row>,denial DenialBinding,
handler identity "worth.query.reference.sign-body.v1",program required,outputs NoApplicationMutationOutputs,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value row,
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
            ParentIdentity::reference(),
            SignedIdentity::reference(),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .signed_by::<SignBinding, _, _, _, _, _, _, _>(
            Actor::reference(),
            Provenance::reference(),
            "path-verification",
        )
        .unwrap()
        .supersede_on_subject_change(
            History::reference(),
            Current::reference(),
            Consequences::reference(),
            Record::reference(),
            std::num::NonZeroUsize::new(8).unwrap(),
        )
        .unwrap()
        .finish()
        .unwrap();
    builder
        .entity(Evidence::reference())
        .aspect(Evidence::reference(), Facts::reference())
        .field(Evidence::reference(), SignedIdentity::reference())
        .field(Evidence::reference(), Actor::reference())
        .field(Evidence::reference(), Provenance::reference())
        .entity(Consequence::reference())
        .aspect(Consequence::reference(), ConsequenceFacts::reference())
        .field(Consequence::reference(), Record::reference())
        .relation(
            History::reference(),
            Row::reference(),
            Evidence::reference(),
        )
        .relation(
            Current::reference(),
            Row::reference(),
            Evidence::reference(),
        )
        .relation(
            Consequences::reference(),
            Evidence::reference(),
            Consequence::reference(),
        )
        .operation(
            SignReference::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        )
        .operation_read_field(SignReference::reference(), RowIdentity::reference())
        .operation_projection_work_budget(SignReference::reference(), 256)
        .application_mutation_binding::<SignBinding>()
}
