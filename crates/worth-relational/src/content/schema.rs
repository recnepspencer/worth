//! Portable format selection is schema authority; executable callbacks are not serialized.
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_foundational::facade::ContentFormat;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RelationalContentSchemaSnapshot {
    formats: Arc<Vec<RelationalContentFormatDeclaration>>,
    fields: Arc<Vec<RelationalContentFieldBinding>>,
    #[serde(skip)]
    _memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalContentFormatDeclaration {
    pub identity: [u8; 32],
    pub descriptor: ContentFormat,
    pub maximum_frame_bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalContentFieldBinding {
    pub field_meaning: [u8; 32],
    pub format: [u8; 32],
}
impl RelationalContentSchemaSnapshot {
    pub(super) fn new(
        formats: Vec<RelationalContentFormatDeclaration>,
        fields: Vec<RelationalContentFieldBinding>,
    ) -> Self {
        Self {
            formats: Arc::new(formats),
            fields: Arc::new(fields),
            _memory: None,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.formats.is_empty() && self.fields.is_empty()
    }
    pub fn formats(&self) -> &[RelationalContentFormatDeclaration] {
        &self.formats
    }
    pub fn fields(&self) -> &[RelationalContentFieldBinding] {
        &self.fields
    }
    pub(crate) fn retain_for_compilation(
        &self,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        use crate::execution::RelationalExecutionDenialCause as Cause;
        if self._memory.is_some()
            || matches!(
                control,
                crate::schema::SchemaCompilationControl::Configuration
            )
        {
            return Ok(self.clone());
        }
        let grant = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<
                worth_execution::ExecutionMemoryReservation,
            >())
            .map_err(|_| Cause::MemoryOverflow)?
            .0
            .pad_to_align()
            .size();
        let bytes = self
            .retained_bytes_for_compilation(control)?
            .checked_add(u64::try_from(grant).map_err(|_| Cause::MemoryOverflow)?)
            .ok_or(Cause::MemoryOverflow)?;
        let memory = control.reserve(usize::try_from(bytes).map_err(|_| Cause::MemoryOverflow)?)?;
        Ok(Self {
            formats: self.formats.clone(),
            fields: self.fields.clone(),
            _memory: memory.map(Arc::new),
        })
    }
    pub(crate) fn retained_bytes_for_compilation(
        &self,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<u64, crate::execution::RelationalExecutionDenialCause> {
        use crate::execution::RelationalExecutionDenialCause as Cause;
        fn roster<T>(capacity: usize) -> Result<usize, Cause> {
            let header = std::alloc::Layout::new::<[usize; 2]>()
                .extend(std::alloc::Layout::new::<Vec<T>>())
                .map_err(|_| Cause::MemoryOverflow)?
                .0
                .pad_to_align()
                .size();
            let data = std::alloc::Layout::array::<T>(capacity)
                .map_err(|_| Cause::MemoryOverflow)?
                .size();
            header.checked_add(data).ok_or(Cause::MemoryOverflow)
        }
        let mut bytes = roster::<RelationalContentFormatDeclaration>(self.formats.capacity())?
            .checked_add(roster::<RelationalContentFieldBinding>(
                self.fields.capacity(),
            )?)
            .ok_or(Cause::MemoryOverflow)?;
        for format in self.formats.iter() {
            control.checkpoint()?;
            bytes = bytes
                .checked_add(format.descriptor.name().len())
                .ok_or(Cause::MemoryOverflow)?;
        }
        u64::try_from(bytes).map_err(|_| Cause::MemoryOverflow)
    }
    pub(crate) fn owned_bytes(&self) -> u64 {
        (self.formats.capacity() * size_of::<RelationalContentFormatDeclaration>()
            + self.fields.capacity() * size_of::<RelationalContentFieldBinding>()
            + 2 * (size_of::<Vec<()>>() + 2 * size_of::<usize>())
            + self
                .formats
                .iter()
                .map(|format| format.descriptor.name().len())
                .sum::<usize>()) as u64
    }
}

impl PartialEq for RelationalContentSchemaSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.formats == other.formats && self.fields == other.fields
    }
}
impl Eq for RelationalContentSchemaSnapshot {}
#[cfg(test)]
#[path = "schema/retention_tests.rs"]
mod retention_tests;
