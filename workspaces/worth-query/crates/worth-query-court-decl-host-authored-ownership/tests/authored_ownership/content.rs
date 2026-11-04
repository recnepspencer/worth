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
worth_query_aspect!(pub Payload for S: Scenario, Node; identity = AspectIdentity(0x919B3943), revision = AspectContractRevision(1));
worth_query_field!(pub Word for S: Scenario, Node, Payload: ContentRefId => ContentRefApplicationValueBinding, read_write, no_equality);
impl OperationWrites<lifecycle::CreateOp> for Word {}
impl OperationWrites<lifecycle::RestoreOp> for Word {}
impl OperationWrites<producer::Operation> for Word {}
impl OperationWrites<coupled::producer::Operation> for Word {}
impl OperationReads<coupled::producer::Operation> for Word {}

pub fn field<S: Scenario>() -> ApplicationContentFieldRef<
    S,
    Node,
    Payload,
    Word,
    ReadWrite,
    NoEqualityPredicate,
    NoApplicationUnit,
    Format,
> {
    content_field(Word::reference(), Format)
}

pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if S::MODE != 43 {
        return schema;
    }
    schema
        .optional_aspect(Node::reference(), Payload::reference())
        .content_format::<Format>()
        .content_field(Node::reference(), field())
        .operation_write(lifecycle::CreateOp::reference(), Word::reference())
        .operation_write(lifecycle::RestoreOp::reference(), Word::reference())
        .operation_write(
            ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration(),
            Word::reference(),
        )
        .operation_write(coupled::producer::Operation::reference(), Word::reference())
        .operation_read_field(coupled::producer::Operation::reference(), Word::reference())
        .operation_projection_work_budget(coupled::producer::Operation::reference(), 512)
}
pub fn read<S: Scenario>(
    reader: &mut DecisionReader<'_, '_, '_, S, coupled::producer::Binding<S>>,
    entity: &WorthQueryInvariantEntityIdentity<S, Node>,
) -> Result<u64, HandlerExecutionDenial> {
    let content = reader
        .content(entity, field())
        .map_err(HandlerExecutionDenial::new)?
        .ok_or_else(|| {
            HandlerExecutionDenial::new(std::io::Error::other("required word content is absent"))
        })?;
    let bytes: [u8; 8] = content.payload().try_into().map_err(|_| {
        HandlerExecutionDenial::new(std::io::Error::other("word content has the wrong shape"))
    })?;
    Ok(u64::from_le_bytes(bytes))
}
