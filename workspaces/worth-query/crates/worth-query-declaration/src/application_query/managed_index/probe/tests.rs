use super::*;
use crate::application_schema::{
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
    BinaryValueDelegate, BinaryValueWriter, CanonicalDateApplicationValueBinding,
    CanonicalTimestampApplicationValueBinding, InlineBytesApplicationValueBinding,
    StringApplicationValueBinding, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds,
};
use worth_foundational::facade::AspectValue;

struct ProbeByte;
struct ProbeByteValue(u8);

impl BinaryValueDelegate for ProbeByte {
    type Value = ProbeByteValue;
    const MAXIMUM_WIRE_BYTES: usize = 1;

    fn encode(
        value: &ProbeByteValue,
        writer: &mut dyn BinaryValueWriter,
    ) -> Result<(), ValueCodecDenial> {
        writer.write(&[value.0])
    }

    fn admit(
        bytes: &[u8],
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<ProbeByteValue, ValueCodecDenial> {
        admission.admit(1, 0)?;
        let [byte] = bytes else {
            return Err(ValueCodecDenial::Malformed);
        };
        Ok(ProbeByteValue(*byte))
    }
}

crate::worth_query_value_binding!(ProbeByteBinding for ProbeByteValue {
    identity: "worth.test.managed-index.probe-byte.v1", binary: ProbeByte,
});

#[derive(worth_foundational::facade::WorthValue)]
struct CanonicalProbe {
    tag: String,
}

crate::worth_query_value_binding!(CanonicalProbeBinding for CanonicalProbe {
    identity: "worth.test.managed-index.canonical-probe.v1",
    scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});

struct UnadmittedNativeBinding;

impl ApplicationScalarValueBinding for UnadmittedNativeBinding {
    type Value = u64;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

    const IDENTITY_NAME: &'static str = "worth.test.managed-index.unadmitted-native.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::UInt64;
    const SUPPORTS_ADMITTED_DECODE: bool = true;

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }

    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::UInt64(*value))
    }
}

impl ApplicationReadableScalarValueBinding for UnadmittedNativeBinding {
    fn decode_admitted(
        value: &AspectValue,
        _: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        Self::decode(value)
    }

    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::UInt64(number) => Ok(*number),
            other => Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY,
                expected: Self::SCALAR_FAMILY,
                observed: other.value_family(),
            }),
        }
    }
}

#[test]
fn admitted_native_scalars_and_binary_binding_are_supported() {
    for probe in [
        ApplicationManagedIndexProbeMeaning::of::<StringApplicationValueBinding>(),
        ApplicationManagedIndexProbeMeaning::of::<InlineBytesApplicationValueBinding>(),
        ApplicationManagedIndexProbeMeaning::of::<CanonicalDateApplicationValueBinding>(),
        ApplicationManagedIndexProbeMeaning::of::<CanonicalTimestampApplicationValueBinding>(),
    ] {
        assert!(probe.parts().canonical.is_none());
        assert!(probe.has_supported_profile());
    }

    let binary = ApplicationManagedIndexProbeMeaning::of::<ProbeByteBinding>();
    assert!(binary.has_supported_profile());
    let mut forged = binary.parts().clone();
    forged.scalar_family = ScalarAspectType::String;
    assert!(
        !ApplicationManagedIndexProbeMeaning::from_untrusted_parts(forged).has_supported_profile()
    );

    for invalid_bytes in [None, Some(0), Some(InlineBytes::MAXIMUM_LENGTH as u64 + 1)] {
        let mut forged = binary.parts().clone();
        let mut canonical = forged.canonical.take().unwrap().parts().clone();
        canonical.maximum_wire_bytes = invalid_bytes;
        forged.canonical = Some(ApplicationCanonicalValueReference::from_untrusted_parts(
            canonical,
        ));
        assert!(
            !ApplicationManagedIndexProbeMeaning::from_untrusted_parts(forged)
                .has_supported_profile()
        );
    }

    let canonical_json = ApplicationManagedIndexProbeMeaning::of::<CanonicalProbeBinding>();
    assert!(canonical_json.has_supported_profile());
    let mut wrong_family = canonical_json.parts().clone();
    wrong_family.scalar_family = ScalarAspectType::InlineBytes;
    assert!(
        !ApplicationManagedIndexProbeMeaning::from_untrusted_parts(wrong_family)
            .has_supported_profile()
    );

    let mut unknown_protocol = canonical_json.parts().clone();
    let mut canonical = unknown_protocol.canonical.take().unwrap().parts().clone();
    canonical.protocol = "worth.unknown.v1".into();
    unknown_protocol.canonical = Some(ApplicationCanonicalValueReference::from_untrusted_parts(
        canonical,
    ));
    assert!(
        !ApplicationManagedIndexProbeMeaning::from_untrusted_parts(unknown_protocol)
            .has_supported_profile()
    );
}

#[test]
fn native_profile_requires_admitted_decode_and_native_admission() {
    let mut parts = ApplicationManagedIndexProbeMeaning::of::<StringApplicationValueBinding>()
        .parts()
        .clone();
    parts.supports_admitted_decode = false;
    assert!(
        !ApplicationManagedIndexProbeMeaning::from_untrusted_parts(parts).has_supported_profile()
    );

    let unadmitted = ApplicationManagedIndexProbeMeaning::of::<UnadmittedNativeBinding>();
    assert!(!unadmitted.parts().supports_admitted_decode);
    assert!(!unadmitted.has_supported_profile());
}
