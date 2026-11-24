//! Registered binary probe binding for public managed-index probe courts.
use worth_query_decl::facade::{application_schema::*, worth_query_value_binding};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TagBytes(pub [u8; 2]);

pub struct TagBytesCodec;

impl BinaryValueDelegate for TagBytesCodec {
    type Value = TagBytes;
    const MAXIMUM_WIRE_BYTES: usize = 2;

    fn encode(
        value: &TagBytes,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&value.0)
    }

    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<TagBytes, ValueCodecDenial> {
        admission.admit(1, 0)?;
        let [a, b] = bytes else {
            return Err(ValueCodecDenial::Malformed);
        };
        Ok(TagBytes([*a, *b]))
    }
}

worth_query_value_binding!(pub TagBytesProbeBinding for TagBytes {
    identity: "managed-probe-watch.tag-bytes.v1",
    binary: TagBytesCodec,
});
