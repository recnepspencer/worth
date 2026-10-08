//! Isolated consumer observation of Foundational's encoding call only.
//! Input creation, allocator metadata, retained storage and tickets are outside
//! this claim; no Query capture or memory admission is exercised here.
#![forbid(unsafe_code)]

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use worth_foundational::facade::{
    write_aspect_value_identity_material, write_struct_aspect_value_identity_material, AspectValue,
    CanonicalBigInt, CanonicalDecimal, CanonicalMaterialByteCount, CanonicalMaterialSink,
    CanonicalRational, FieldKey, StructAspectValue,
};

#[global_allocator]
static TEST_ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

const PROBE_ENV: &str = "WORTH_FOUNDATIONAL_BORROWED_CANONICAL_ALLOCATION_PROBE";

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
        AspectValue::String("q".repeat(128 * 1024).into()),
        AspectValue::Decimal(CanonicalDecimal("1234567890".repeat(1024))),
        AspectValue::BigInt(CanonicalBigInt("1234567890".repeat(1024))),
        AspectValue::Rational(CanonicalRational {
            numerator: CanonicalBigInt("1234567890".repeat(1024)),
            denominator: CanonicalBigInt("9876543210".repeat(1024)),
        }),
    ];
    let structure = StructAspectValue::new([
        (FieldKey::new("body").unwrap(), inputs[0].clone()),
        (FieldKey::new("ratio").unwrap(), inputs[3].clone()),
    ])
    .unwrap();
    let mut sink = StackSink {
        bytes: [0; 256 * 1024],
        length: 0,
    };
    for input in &inputs {
        sink.length = 0;
        let mut count = CanonicalMaterialByteCount::new();
        let region = Region::new(&INSTRUMENTED_SYSTEM);
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
    let mut count = CanonicalMaterialByteCount::new();
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let writing = write_struct_aspect_value_identity_material(&structure, &mut sink);
    let counting = write_struct_aspect_value_identity_material(&structure, &mut count);
    let observed = region.change();
    writing.unwrap();
    counting.unwrap();
    assert_eq!(observed.allocations, 0);
    assert_eq!(observed.reallocations, 0);
    assert_eq!(count.encoded_bytes(), sink.length);
}

struct StackSink {
    bytes: [u8; 256 * 1024],
    length: usize,
}

impl CanonicalMaterialSink for StackSink {
    type Error = ();
    fn append(&mut self, value: &str) -> Result<(), ()> {
        let end = self.length.checked_add(value.len()).ok_or(())?;
        self.bytes
            .get_mut(self.length..end)
            .ok_or(())?
            .copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
    fn admit_work(&mut self, _: usize) -> Result<(), ()> {
        Ok(())
    }
    fn accounting_overflow(&mut self) {}
}
