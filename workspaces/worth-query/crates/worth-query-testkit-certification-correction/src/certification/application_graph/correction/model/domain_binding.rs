//! Versioned opaque counters exercise real correction and Native scalar owners.
use worth_query_decl::facade::application_schema::{
    BinaryValueDelegate, BinaryValueWriter, ValueCodecDenial, ValueDecodeAdmission,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Counter(pub u64);
pub struct CounterDelegate;
impl BinaryValueDelegate for CounterDelegate {
    type Value = Counter;
    const MAXIMUM_WIRE_BYTES: usize = 9;
    fn encode(value: &Counter, output: &mut dyn BinaryValueWriter) -> Result<(), ValueCodecDenial> {
        output.write(&[1])?;
        output.write(&value.0.to_le_bytes())
    }
    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Counter, ValueCodecDenial> {
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
        Ok(Counter(u64::from_le_bytes([
            *a, *b, *c, *d, *e, *f, *g, *h,
        ])))
    }
}
worth_query_decl::facade::worth_query_value_binding!(pub InlineCounterBinding for Counter {
    identity: "numeric-correction.inline-counter.v1", binary: CounterDelegate,
    frame: "fixture.frames.model.v1",
});
