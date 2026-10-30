// Isolated consumer observation of Foundational's encoding call only.
// Input creation, allocator metadata, retained storage and tickets are outside
// this claim; no Query capture or memory admission is exercised here.
#![forbid(unsafe_code)]

pub(crate) use {stats_alloc::Region, stats_alloc::StatsAlloc, stats_alloc::INSTRUMENTED_SYSTEM};
pub(crate) use {
    worth_foundational::facade::write_aspect_value_identity_material,
    worth_foundational::facade::write_struct_aspect_value_identity_material,
    worth_foundational::facade::AspectValue, worth_foundational::facade::CanonicalBigInt,
    worth_foundational::facade::CanonicalDecimal,
    worth_foundational::facade::CanonicalMaterialByteCount,
    worth_foundational::facade::CanonicalMaterialSink,
    worth_foundational::facade::CanonicalRational, worth_foundational::facade::FieldKey,
    worth_foundational::facade::StructAspectValue,
};

#[global_allocator]
pub(crate) static TEST_ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

pub(crate) const PROBE_ENV: &str = "WORTH_FOUNDATIONAL_BORROWED_CANONICAL_ALLOCATION_PROBE";

pub(crate) struct StackSink {
    pub(crate) bytes: [u8; 256 * 1024],
    pub(crate) length: usize,
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
