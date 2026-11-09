//! Content retains its actual field and Native byte custody through writer succession.
use super::*;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
mod courts;

#[derive(Clone, Copy)]
pub struct Format;
impl ApplicationContentBinding for Format {
    type Value = u64;
    const IDENTITY: CanonicalDigestId = CanonicalDigestId::new([143; 32]);
    const NAME: &'static str = "ownership-exact-word";
    const VERSION: u32 = 1;
    const MAXIMUM_FRAME_BYTES: usize = 64;
    fn payload_length(
        _: &u64,
        control: &mut dyn ApplicationContentControl,
    ) -> Result<usize, ApplicationContentCodecDenial> {
        control.checkpoint(1)?;
        Ok(8)
    }
    fn encode(
        value: &u64,
        out: &mut dyn ApplicationContentEncoder,
    ) -> Result<(), ApplicationContentCodecDenial> {
        out.append(&value.to_le_bytes())
    }
    fn validate(
        bytes: &[u8],
        control: &mut dyn ApplicationContentControl,
    ) -> Result<(), ApplicationContentCodecDenial> {
        control.checkpoint(bytes.len() as u64)?;
        if bytes.len() == 8 {
            Ok(())
        } else {
            Err(ApplicationContentCodecDenial::NonCanonicalContent)
        }
    }
    fn verify_encoding(
        value: &u64,
        bytes: &[u8],
        control: &mut dyn ApplicationContentControl,
    ) -> Result<(), ApplicationContentCodecDenial> {
        control.checkpoint(8)?;
        let word: [u8; 8] = bytes
            .try_into()
            .map_err(|_| ApplicationContentCodecDenial::EncodingInconsistency)?;
        if u64::from_le_bytes(word) == *value {
            Ok(())
        } else {
            Err(ApplicationContentCodecDenial::EncodingInconsistency)
        }
    }
}
impl<const CONTENT: bool> OperationWrites<lifecycle::CreateOp> for RawWord<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<lifecycle::RestoreOp> for RawWord<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<producer::Operation> for RawWord<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<coupled::producer::Operation> for RawWord<CONTENT> {}
impl<const CONTENT: bool> OperationReads<coupled::producer::Operation> for RawWord<CONTENT> {}

pub fn field<S: Scenario>() -> Option<ApplicationContentFieldRef<Word<S>, Format>> {
    <S::NodeProfile as schema::node_profile::NodeProfile<S>>::content()
}

pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if S::MODE != 43 {
        return schema;
    }
    schema
        .operation_write(
            lifecycle::CreateOp::reference(),
            <S::NodeProfile as schema::node_profile::NodeProfile<S>>::word()
                .expect("the content scenario declares its field"),
        )
        .operation_write(
            lifecycle::RestoreOp::reference(),
            <S::NodeProfile as schema::node_profile::NodeProfile<S>>::word()
                .expect("the content scenario declares its field"),
        )
        .operation_write(
            ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
            <S::NodeProfile as schema::node_profile::NodeProfile<S>>::word()
                .expect("the content scenario declares its field"),
        )
        .operation_write(
            coupled::producer::Operation::reference(),
            <S::NodeProfile as schema::node_profile::NodeProfile<S>>::word()
                .expect("the content scenario declares its field"),
        )
        .operation_read_field(
            coupled::producer::Operation::reference(),
            <S::NodeProfile as schema::node_profile::NodeProfile<S>>::word()
                .expect("the content scenario declares its field"),
        )
        .operation_projection_work_budget(coupled::producer::Operation::reference(), 512)
}
pub fn read<S: Scenario>(
    reader: &mut DecisionReader<'_, '_, '_, S, coupled::producer::Binding<S>>,
    entity: &WorthQueryInvariantEntityIdentity<S, Node<S>>,
) -> Result<u64, HandlerExecutionDenial> {
    let content = reader
        .content(
            entity,
            field::<S>().ok_or_else(|| {
                HandlerExecutionDenial::new(std::io::Error::other(
                    "content is not declared by this scenario",
                ))
            })?,
        )
        .map_err(HandlerExecutionDenial::new)?
        .ok_or_else(|| {
            HandlerExecutionDenial::new(std::io::Error::other("required word content is absent"))
        })?;
    let bytes: [u8; 8] = content.payload().try_into().map_err(|_| {
        HandlerExecutionDenial::new(std::io::Error::other("word content has the wrong shape"))
    })?;
    Ok(u64::from_le_bytes(bytes))
}

use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct Payload;
impl Aspect for Payload {
    const NAME: &'static str = "Payload";
    type Row = WordRow;
}
pub type RawWord<const CONTENT: bool> =
    FieldSelection<OwnershipModel<CONTENT>, NativeNode<CONTENT>, Payload, ContentRefId>;
pub type Word<S> = <<S as Scenario>::NodeProfile as schema::node_profile::NodeProfile<S>>::Word;
impl Scalar<ContentRefId> for OwnershipModelBindings {
    type Binding = ContentRefApplicationValueBinding;
    fn content_format() -> Option<ApplicationContentBindingRecipe> {
        Some(ApplicationContentBindingRecipe::of::<Format>())
    }
}

pub struct WordRow(ContentRefId);
impl fragment::Record<WordRow> for OwnershipModelBindings {
    fn record() -> impl Shape<WordRow, Self> {
        fragment::field("Word", |row: &WordRow| row.0).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(WordRow(value))
            },
        )
    }
}
