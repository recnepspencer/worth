//! A stale encoder emits version two while its reader admits only version one.
use worth_query_decl::facade::application_schema::{
    BinaryValueDelegate, BinaryValueWriter, ValueCodecDenial, ValueDecodeAdmission,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BinaryCounter(pub u64);
pub struct StaleEncoder;
impl BinaryValueDelegate for StaleEncoder {
    type Value = BinaryCounter;
    const MAXIMUM_WIRE_BYTES: usize = 9;
    fn encode(
        value: &BinaryCounter,
        output: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        output.write(&[if value.0 == 13 { 2 } else { 1 }])?;
        output.write(&value.0.to_le_bytes())
    }
    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<BinaryCounter, ValueCodecDenial> {
        let [version, a, b, c, d, e, f, g, h] = bytes else {
            return Err(ValueCodecDenial::Malformed);
        };
        if *version != 1 {
            return Err(ValueCodecDenial::UnsupportedVersion {
                expected: 1,
                observed: u32::from(*version),
            });
        }
        let value = u64::from_le_bytes([*a, *b, *c, *d, *e, *f, *g, *h]);
        let result = match value {
            17 => admission.admit(1_000_000, 0),
            19 => admission.admit(1, 8 * 1024 * 1024),
            _ => admission.admit(1, 0),
        };
        // Malicious owner masks the request stop with an ordinary codec error.
        // The original request refusal must still win at publication.
        if result.is_err() {
            return Err(ValueCodecDenial::Malformed);
        }
        Ok(BinaryCounter(value))
    }
}
worth_query_decl::facade::worth_query_value_binding!(pub BrokenBinaryBinding for BinaryCounter {
    identity: "domain-write.stale-binary.v1", binary: StaleEncoder,
});
