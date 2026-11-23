use super::*;
use crate::application_schema::{
    BinaryValueDelegate, BinaryValueWriter, CanonicalDateApplicationValueBinding,
    CanonicalTimestampApplicationValueBinding, InlineBytesApplicationValueBinding,
    StringApplicationValueBinding, ValueCodecDenial, ValueDecodeAdmission,
};

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

    let mut forged = binary.parts().clone();
    let mut canonical = forged.canonical.take().unwrap().parts().clone();
    canonical.maximum_wire_bytes = None;
    forged.canonical = Some(ApplicationCanonicalValueReference::from_untrusted_parts(
        canonical,
    ));
    assert!(
        !ApplicationManagedIndexProbeMeaning::from_untrusted_parts(forged).has_supported_profile()
    );
}

#[test]
fn native_profile_requires_admitted_decode() {
    let mut parts = ApplicationManagedIndexProbeMeaning::of::<StringApplicationValueBinding>()
        .parts()
        .clone();
    parts.supports_admitted_decode = false;
    assert!(
        !ApplicationManagedIndexProbeMeaning::from_untrusted_parts(parts).has_supported_profile()
    );
}
