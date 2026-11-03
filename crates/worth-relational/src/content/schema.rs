//! Portable format selection is schema authority; executable callbacks are not serialized.
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_foundational::facade::ContentFormat;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalContentSchemaSnapshot {
    formats: Arc<Vec<RelationalContentFormatDeclaration>>,
    fields: Arc<Vec<RelationalContentFieldBinding>>,
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
