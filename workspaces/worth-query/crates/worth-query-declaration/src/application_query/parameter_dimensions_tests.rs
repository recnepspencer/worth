use super::{ApplicationQueryParameterDefinition, ApplicationQueryParameterRef};

struct Distance(u64);
impl Distance {
    fn encode(value: &Self) -> u64 {
        value.0
    }
    fn decode(value: u64) -> Option<Self> {
        Some(Self(value))
    }
}
crate::worth_query_value_binding! {
    DistanceBinding for Distance {
        identity: "worth.tests.distance.v1",
        scalar: UInt64,
        unit: "worth.units.metre.v1",
        frame: "worth.frames.model.v1",
        encode: Distance::encode,
        decode: Distance::decode,
    }
}

#[test]
fn parameter_description_derives_dimension_metadata_from_the_actual_scalar_binding() {
    let parameter =
        ApplicationQueryParameterDefinition::typed(ApplicationQueryParameterRef::<
            (),
            (),
            DistanceBinding,
        >::from_query_identifier("distance"));
    assert_eq!(parameter.value_type(), "worth.tests.distance.v1");
    assert_eq!(parameter.unit(), Some("worth.units.metre.v1"));
    assert_eq!(parameter.frame(), Some("worth.frames.model.v1"));
}
