//! Recorded declarations never manufacture executable format handles.
use super::{Denial, RelationalContentFormatRegistry};
use crate::content::{RelationalContentSchemaSnapshot, MAXIMUM_CONTENT_FRAME_BYTES};
use worth_foundational::facade::CanonicalDigestId;

impl RelationalContentFormatRegistry {
    /// Descriptive carrier validation only. No format handle can be resolved.
    pub(crate) fn from_recorded(snapshot: RelationalContentSchemaSnapshot) -> Result<Self, Denial> {
        let mut registry = Self::default();
        let mut previous = None;
        for format in snapshot.formats() {
            format
                .descriptor
                .validate()
                .map_err(|_| Denial::InvalidFormat)?;
            if previous.is_some_and(|prior| prior >= format.identity) {
                return Err(Denial::NonCanonicalContent);
            }
            previous = Some(format.identity);
            if format.maximum_frame_bytes > MAXIMUM_CONTENT_FRAME_BYTES as u64 {
                return Err(Denial::SizeLimit {
                    observed: usize::try_from(format.maximum_frame_bytes)
                        .map_err(|_| Denial::ArithmeticOverflow)?,
                    maximum: MAXIMUM_CONTENT_FRAME_BYTES,
                });
            }
            let identity = CanonicalDigestId::new(format.identity);
            if registry
                .names
                .entry(format.descriptor.name().to_owned())
                .or_default()
                .insert(format.descriptor.version(), identity)
                .is_some()
            {
                return Err(Denial::DuplicateContentFormat);
            }
        }
        let mut previous = None;
        for field in snapshot.fields() {
            if previous.is_some_and(|prior| prior >= field.field_meaning) {
                return Err(Denial::NonCanonicalContent);
            }
            previous = Some(field.field_meaning);
            if snapshot
                .formats()
                .binary_search_by_key(&field.format, |format| format.identity)
                .is_err()
            {
                return Err(Denial::FormatBindingMissing);
            }
            registry.bindings.insert(
                CanonicalDigestId::new(field.field_meaning),
                CanonicalDigestId::new(field.format),
            );
        }
        registry.snapshot = snapshot;
        Ok(registry)
    }
    /// Bind recorded meaning to actual configured frozen handles before use.
    pub(crate) fn bind_recorded(&self, available: &Self) -> Result<Self, Denial> {
        let mut bound = Self::from_recorded(self.snapshot.clone())?;
        for declaration in self.snapshot.formats() {
            let identity = CanonicalDigestId::new(declaration.identity);
            let installed = available.get(identity)?;
            if installed.descriptor().name() != declaration.descriptor.name() {
                return Err(Denial::FormatMismatch);
            }
            if installed.descriptor().version() != declaration.descriptor.version() {
                return Err(Denial::UnsupportedVersion);
            }
            if installed.maximum_frame_bytes() as u64 != declaration.maximum_frame_bytes {
                return Err(Denial::FormatMismatch);
            }
            bound.formats.insert(identity, installed.clone());
        }
        Ok(bound)
    }
}
