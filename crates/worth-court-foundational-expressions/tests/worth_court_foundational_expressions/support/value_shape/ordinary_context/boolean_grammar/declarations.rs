// Actual array aliases, platform indices and named enum payloads share one wire grammar.
pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};
pub(crate) use {worth_foundational::facade::decode_canonical_value, worth_foundational::facade::decode_canonical_value_with_admission, worth_foundational::facade::encode_canonical_value, worth_foundational::facade::CanonicalValue, worth_foundational::facade::ValueCodecDenial as D, worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::WorthValue};

pub(crate) type Entity = [u64; 3];
pub(crate) type Nodes = [[usize; 3]; 2];
#[derive(Debug, PartialEq, WorthValue)]
pub(crate) enum Meaning {
    Empty,
    Name(String),
    Face {
        #[serde(rename = "source")]
        entity: Entity,
        nodes: Nodes,
        contributors: Vec<(usize, [u64; 6])>,
    },
}
pub(crate) const BOUNDS: ValueDecodeBounds = worth_foundational::facade::ValueDecodeBounds::new(4096, 256, 64, 32);
pub(crate) const RAW: &str = r#"{"Face":{"source":[9,17,4],"nodes":[[1,2,3],[4,5,6]],"contributors":[[7,[11,12,13,14,15,16]]]}}"#;

pub(crate) static DROPPED: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[derive(Debug, WorthValue)]
pub(crate) struct Element(pub(crate) u64);
impl Drop for Element {
    fn drop(&mut self) {
        DROPPED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub(crate) struct DomainValue(pub(crate) u64);
impl DomainValue {
    pub(crate) fn try_new(value: u64) -> Result<Self, D> {
        if value == 0 {
            Err(worth_foundational::facade::ValueCodecDenial::NonCanonical)
        } else {
            Ok(Self(value))
        }
    }
}
pub(crate) struct DomainDelegate;
impl worth_foundational::facade::ValueSeedDelegate for DomainDelegate {
    type Value = DomainValue;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: worth_foundational::facade::ValueShapeBody =
        <u64 as CanonicalValue>::SHAPE_BODY;
    fn validate(value: &DomainValue, bounds: ValueDecodeBounds) -> Result<(), D> {
        crate::value_shape::ordinary_context::boolean_grammar::DomainValue::try_new(value.0)?;
        value.0.validate_bounds(bounds)
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        mut admission: Option<&mut worth_foundational::facade::ValueDecodeAdmission<'_>>,
    ) -> Result<DomainValue, Decoder::Error> {
        let value = serde::de::DeserializeSeed::deserialize(
            worth_foundational::facade::CanonicalValueSeed::<u64>::with_optional_admission(
                bounds,
                admission.as_deref_mut(),
            ),
            decoder,
        )?;
        crate::value_shape::ordinary_context::boolean_grammar::DomainValue::try_new(value)
            .map_err(|cause| worth_foundational::facade::value_decode_error(cause, admission))
    }
}
#[derive(Debug, PartialEq, WorthValue)]
pub(crate) struct Delegated {
    #[worth_value(delegate = DomainDelegate)]
    pub(crate) chosen: DomainValue,
}