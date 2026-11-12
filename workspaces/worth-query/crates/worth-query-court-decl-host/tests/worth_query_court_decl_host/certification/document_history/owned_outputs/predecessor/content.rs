//! Content is read through the actual independently authorized predecessor loan.
use super::*;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_query_decl::facade::application_schema::fragment::*;
mod codec;
mod courts;
use codec::Word;
pub struct Payload;
impl Aspect for Payload {
    const NAME: &'static str = "Payload";
    type Row = PayloadRow;
}
pub struct PayloadRow {
    pub value: ContentRefId,
}
impl Scalar<ContentRefId> for Bindings {
    type Binding = ContentRefApplicationValueBinding;
    fn content_format() -> Option<ApplicationContentBindingRecipe> {
        Some(ApplicationContentBindingRecipe::of::<Word>())
    }
}
impl Record<PayloadRow> for Bindings {
    fn record() -> impl Shape<PayloadRow, Self> {
        field("Payload", |row: &PayloadRow| row.value).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PayloadRow { value })
            },
        )
    }
}
pub type Values = FieldSelection<Storage, Model, Payload, ContentRefId>;
fn payload() -> ApplicationContentFieldRef<Values, Word> {
    content_field(Values::at("Payload"), Word)
}
worth_query_operation!(pub ReadContentInputs for Schema, input query::ParametersBinding; access {
    read_entities: [Model]; read_fields: [ModelValue::at("ModelValue"), Values::at("Payload")];
});
worth_query_operation_requires!(ReadContentInputs => [ReadInputs]);
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<ReadContentInputs>(
            ReadContentInputs::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_requires_ability(ReadContentInputs::reference(), ReadInputs::reference())
}
pub(in super::super) fn prepare(
    reader: &mut DecisionReader<'_, '_, '_, Schema, edit::Binding>,
    entity: &WorthQueryInvariantEntityIdentity<Schema, Model>,
) -> Result<(), HandlerExecutionDenial> {
    if PROBE.with(|probe| probe.borrow().content_mode.is_some()) {
        reader.field(entity, Values::at("Payload"))?;
    }
    Ok(())
}
pub(in super::super) fn write(
    writer: &mut CandidateWriter<'_, Schema, edit::Binding>,
    entity: &WorthQueryApplicationEffectEntity<Schema, Model>,
    value: u64,
) -> Result<(), ContentWriteDenial> {
    if PROBE.with(|probe| probe.borrow().content_mode.is_some()) {
        writer.write_content(entity, payload(), &value)?;
    }
    Ok(())
}
pub(super) fn observe<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    source: &WorthQueryInvariantEntityIdentity<Schema, Model>,
    granted: bool,
) -> ReadResult {
    if granted {
        reader.predecessor_observation(ReadContentInputs::reference(), |old| read(old, source))
    } else {
        reader.predecessor_observation(ReadOldInputs::reference(), |old| read(old, source))
    }
}
fn read<O>(
    old: &mut PredecessorDecisionReader<'_, '_, '_, Schema, O>,
    source: &WorthQueryInvariantEntityIdentity<Schema, Model>,
) -> Option<u64>
where
    Model: OperationReads<O>,
{
    let entity = old.entity(Model::reference(), source).unwrap();
    let result = old.content(&entity, payload());
    PROBE.with(|probe| {
        let mut probe = probe.borrow_mut();
        match result {
            Ok(value) => probe
                .content_values
                .push(value.map(|content| content.payload().to_vec())),
            Err(ContentReadDenial::Field(_)) => probe.content_denied = true,
            Err(other) => panic!("a forbidden field must keep its original denial: {other:?}"),
        }
    });
    // A field-permission refusal exposes no bytes; the declared scalar remains usable.
    old.field(&entity, ModelValue::at("ModelValue")).unwrap()
}
