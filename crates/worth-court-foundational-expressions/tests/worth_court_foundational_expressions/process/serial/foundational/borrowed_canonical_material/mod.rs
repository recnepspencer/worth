// Isolated consumer observation of Foundational's encoding call only.
// Input creation, allocator metadata, retained storage and tickets are outside
// this claim; no Query capture or memory admission is exercised here.
#![forbid(unsafe_code)]

#[allow(unused_imports)] use crate::borrowed_canonical_material::isolated_context::*;
use {stats_alloc::Region, stats_alloc::StatsAlloc, stats_alloc::INSTRUMENTED_SYSTEM};
use {worth_foundational::facade::write_aspect_value_identity_material, worth_foundational::facade::write_struct_aspect_value_identity_material, worth_foundational::facade::AspectValue, worth_foundational::facade::CanonicalBigInt, worth_foundational::facade::CanonicalDecimal, worth_foundational::facade::CanonicalMaterialByteCount, worth_foundational::facade::CanonicalMaterialSink, worth_foundational::facade::CanonicalRational, worth_foundational::facade::FieldKey, worth_foundational::facade::StructAspectValue};

#[test]
fn borrowed_canonical_writing_allocates_no_intermediate_buffers() {
    let filter = concat!(
        module_path!(),
        "::isolated_borrowed_canonical_writing_probe"
    )
    .split_once("::")
    .unwrap()
    .1;
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", filter, "--test-threads=1", "--nocapture"])
        .env(PROBE_ENV, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}

#[test]
fn isolated_borrowed_canonical_writing_probe() {
    if std::env::var_os(PROBE_ENV).is_none() {
        return;
    }
    let inputs = [
        worth_foundational::facade::AspectValue::String("q".repeat(128 * 1024).into()),
        worth_foundational::facade::AspectValue::Decimal(CanonicalDecimal("1234567890".repeat(1024))),
        worth_foundational::facade::AspectValue::BigInt(CanonicalBigInt("1234567890".repeat(1024))),
        worth_foundational::facade::AspectValue::Rational(CanonicalRational {
            numerator: CanonicalBigInt("1234567890".repeat(1024)),
            denominator: CanonicalBigInt("9876543210".repeat(1024)),
        }),
    ];
    let structure = worth_foundational::facade::StructAspectValue::new([
        (worth_foundational::facade::FieldKey::new("body").unwrap(), inputs[0].clone()),
        (worth_foundational::facade::FieldKey::new("ratio").unwrap(), inputs[3].clone()),
    ])
    .unwrap();
    let mut sink = StackSink {
        bytes: [0; 256 * 1024],
        length: 0,
    };
    for input in &inputs {
        sink.length = 0;
        let mut count = worth_foundational::facade::CanonicalMaterialByteCount::new();
        let region = stats_alloc::Region::new(&INSTRUMENTED_SYSTEM);
        let writing = write_aspect_value_identity_material(input, &mut sink);
        let counting = write_aspect_value_identity_material(input, &mut count);
        let observed = region.change();
        writing.unwrap();
        counting.unwrap();
        assert_eq!(observed.allocations, 0);
        assert_eq!(observed.reallocations, 0);
        assert_eq!(count.encoded_bytes(), sink.length);
    }
    sink.length = 0;
    let mut count = worth_foundational::facade::CanonicalMaterialByteCount::new();
    let region = stats_alloc::Region::new(&INSTRUMENTED_SYSTEM);
    let writing = write_struct_aspect_value_identity_material(&structure, &mut sink);
    let counting = write_struct_aspect_value_identity_material(&structure, &mut count);
    let observed = region.change();
    writing.unwrap();
    counting.unwrap();
    assert_eq!(observed.allocations, 0);
    assert_eq!(observed.reallocations, 0);
    assert_eq!(count.encoded_bytes(), sink.length);
}
