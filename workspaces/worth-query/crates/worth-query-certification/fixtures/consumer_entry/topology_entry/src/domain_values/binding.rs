pub use worth_query_consumer_values::Ratio;
use worth_query_decl::facade::application_schema::{
    BinaryValueDelegate, BinaryValueWriter, ValueCodecDenial as D, ValueDecodeAdmission,
};

pub struct RatioDelegate;
impl BinaryValueDelegate for RatioDelegate {
    type Value = Ratio;
    const MAXIMUM_WIRE_BYTES: usize = 9;
    fn encode(value: &Ratio, output: &mut dyn BinaryValueWriter) -> Result<(), D> {
        let mut bytes = [0; 9];
        bytes[0] = 1;
        bytes[1..5].copy_from_slice(&value.numerator().to_le_bytes());
        bytes[5..9].copy_from_slice(&value.denominator().to_le_bytes());
        output.write(&bytes)
    }
    fn admit(raw: &[u8], admission: &mut ValueDecodeAdmission<'_>) -> Result<Ratio, D> {
        let [version, a, b, c, d, e, f, g, h] = raw else {
            return Err(D::Malformed);
        };
        if *version != 1 {
            return Err(D::UnsupportedVersion {
                expected: 1,
                observed: u32::from(*version),
            });
        }
        let numerator = u32::from_le_bytes([*a, *b, *c, *d]);
        let denominator = u32::from_le_bytes([*e, *f, *g, *h]);
        // u32 Euclid needs fewer than 48 divisions, plus a fixed certificate
        // check. This original grant covers construction before it runs.
        admission.admit(128, 0)?;
        let value = Ratio::new(numerator, denominator).map_err(|cause| match cause {
            worth_query_consumer_values::RatioDenial::ZeroDenominator => D::Malformed,
            worth_query_consumer_values::RatioDenial::InternalInconsistency => {
                D::InternalInconsistency
            }
        })?;
        if value.numerator() != numerator || value.denominator() != denominator {
            return Err(D::NonCanonical);
        }
        Ok(value)
    }
}

worth_query_decl::facade::worth_query_value_binding!(pub RatioBinding for worth_query_consumer_values::Ratio {
    identity: "fixture.ratio.v1", binary: RatioDelegate,
    unit: "fixture.unit.ratio.v1", frame: "fixture.frames.model.v1",
});

#[derive(Clone, Debug, Eq, PartialEq, worth_query_decl::facade::application_schema::WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Span {
    pub length: u32,
}

worth_query_decl::facade::worth_query_value_binding!(pub SpanBinding for Span {
    identity: "fixture.span.v1", scalar: String,
    canonical: worth_query_decl::facade::application_schema::ValueDecodeBounds::new(21, 6, 1, 2), domain_value,
    unit: "fixture.unit.length.v1", frame: "fixture.frames.model.v1",
});

worth_query_decl::facade::worth_query_value_binding!(pub StructuredSpanBinding for Span {
    identity: "fixture.structured-span.v1", scalar: String, canonical,
    unit: "fixture.unit.length.v1", frame: "fixture.frames.model.v1",
});
