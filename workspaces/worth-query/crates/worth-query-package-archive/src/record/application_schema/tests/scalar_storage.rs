//! Append the scalar-storage qualifier without reinterpreting existing member tags.
use super::*;
#[test]
fn scalar_storage_member_round_trips_without_changing_struct_aspect_encoding() {
    let scalar = ApplicationSchemaMember::ScalarAspectField {
        entity: "Record".into(),
        aspect: "Facts".into(),
        field: "Value".into(),
    };
    let bytes = encode_member(&scalar);
    assert_eq!(&bytes[..2], &[0, 50]);
    assert_eq!(decode_member(&bytes), scalar);
    let structure = ApplicationSchemaMember::Aspect {
        entity: "Record".into(),
        aspect: "Facts".into(),
        identity: worth_foundational::facade::AspectIdentity(71),
        revision: worth_foundational::facade::AspectContractRevision(1),
        absence: worth_foundational::facade::AbsenceLaw::Required,
    };
    let old = encode_member(&structure);
    assert_eq!(&old[..2], &[0, 2]);
    assert_eq!(decode_member(&old), structure);
}
