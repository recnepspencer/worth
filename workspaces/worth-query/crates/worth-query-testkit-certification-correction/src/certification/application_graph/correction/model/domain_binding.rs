//! Versioned opaque counters exercise real correction and Native scalar owners.
use worth_query_decl::facade::application_schema::{
    BinaryValueDelegate, BinaryValueWriter, ValueCodecDenial, ValueDecodeAdmission,
};

pub struct CounterDelegate;
impl BinaryValueDelegate for CounterDelegate {
    type Value = u64;
    const MAXIMUM_WIRE_BYTES: usize = 9;
    fn encode(value: &u64, output: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        output.write(&[1])?;
        output.write(&value.to_le_bytes())
    }
    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<u64, ValueCodecDenial> {
        let [version, a, b, c, d, e, f, g, h] = bytes else {
            return Err(ValueCodecDenial::Malformed);
        };
        if *version != 1 {
            return Err(ValueCodecDenial::UnsupportedVersion {
                expected: 1,
                observed: u32::from(*version),
            });
        }
        admission.admit(1, 0)?;
        Ok(u64::from_le_bytes([*a, *b, *c, *d, *e, *f, *g, *h]))
    }
}
worth_query_decl::facade::worth_query_value_binding!(pub InlineCounterBinding for u64 {
    identity: "numeric-correction.inline-counter.v1", binary: CounterDelegate,
    frame: "fixture.frames.model.v1",
});
