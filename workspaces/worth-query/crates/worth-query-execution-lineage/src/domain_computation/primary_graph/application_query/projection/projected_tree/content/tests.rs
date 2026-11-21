//! Framing/value representation only; none of these values is Source authority.
use super::*;
fn material(parts: &[&[u8]], scalar: &AspectValue, generation: u32) -> [u8; 32] {
    let mut admit = |_| Ok::<_, ()>(());
    let mut writer = ContentWriter::new(&mut admit, &|| ());
    writer
        .entity(EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            7,
            generation,
        ))
        .unwrap();
    for part in parts {
        writer.bytes(part).unwrap();
    }
    writer.scalar(scalar).unwrap();
    writer.finish()
}
#[test]
fn framed_native_content_distinguishes_field_boundaries_values_types_and_generations() {
    let shared = AspectValue::String("shared".into());
    let reference = material(&[b"name", b"enabled"], &shared, 1);
    assert_eq!(reference, material(&[b"name", b"enabled"], &shared, 1));
    assert_ne!(
        material(&[b"a", b"bc"], &shared, 1),
        material(&[b"ab", b"c"], &shared, 1)
    );
    assert_ne!(
        reference,
        material(
            &[b"name", b"enabled"],
            &AspectValue::String("sham".into()),
            1
        )
    );
    assert_ne!(reference, material(&[b"name", b"enabled"], &shared, 2));
    assert_ne!(
        material(&[b"value"], &AspectValue::Bool(true), 1),
        material(&[b"value"], &AspectValue::String("true".into()), 1)
    );
}
#[test]
fn canonical_native_scalar_failure_propagates_owned_stop_without_intermediate_text() {
    #[derive(Debug, Eq, PartialEq)]
    struct Stop(&'static str);
    let value = AspectValue::String("a\0b-é".repeat(2048).into());
    let mut contacts = 0;
    let mut admit = |_| {
        contacts += 1;
        Err(Stop("original-paid-sink-stop"))
    };
    let mut writer = ContentWriter::new(&mut admit, &|| Stop("overflow"));
    assert_eq!(writer.scalar(&value), Err(Stop("original-paid-sink-stop")));
    assert_eq!(contacts, 1);
}
