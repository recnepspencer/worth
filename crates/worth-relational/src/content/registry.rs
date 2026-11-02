//! Format admission and field-binding aliases are separate schema coordinates.
mod readmission;
use super::{RelationalContentCodec as Codec, RelationalContentDenial as Denial};
use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::{CanonicalDigestId, ContentFormat};

#[derive(Clone, Debug)]
pub struct RelationalContentFormat(Arc<InstalledFormat>);
#[derive(Debug)]
struct InstalledFormat {
    descriptor: ContentFormat,
    maximum: usize,
    codec: Arc<dyn Codec>,
}
impl RelationalContentFormat {
    pub(super) fn same_installation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn descriptor(&self) -> &ContentFormat {
        &self.0.descriptor
    }
    pub fn maximum_frame_bytes(&self) -> usize {
        self.0.maximum
    }
    pub(super) fn codec(&self) -> &dyn Codec {
        self.0.codec.as_ref()
    }
}

#[derive(Clone, Debug, Default)]
pub struct RelationalContentFormatRegistry {
    snapshot: super::RelationalContentSchemaSnapshot,
    formats: BTreeMap<CanonicalDigestId, RelationalContentFormat>,
    names: BTreeMap<String, BTreeMap<u32, CanonicalDigestId>>,
    bindings: BTreeMap<CanonicalDigestId, CanonicalDigestId>,
}
impl PartialEq for RelationalContentFormatRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot == other.snapshot
            && self.names == other.names
            && self.bindings == other.bindings
            && self.formats.len() == other.formats.len()
            && self.formats.iter().all(|(key, format)| {
                other
                    .formats
                    .get(key)
                    .is_some_and(|other| Arc::ptr_eq(&format.0, &other.0))
            })
    }
}
impl Eq for RelationalContentFormatRegistry {}
impl RelationalContentFormatRegistry {
    /// Freeze executable configuration once. A later alias reuses this handle.
    pub fn register(
        &mut self,
        identity: CanonicalDigestId,
        codec: Arc<dyn Codec>,
    ) -> Result<(), Denial> {
        let descriptor = codec.format().clone();
        descriptor.validate().map_err(|_| Denial::InvalidFormat)?;
        let maximum = codec
            .maximum_frame_bytes()
            .min(super::MAXIMUM_CONTENT_FRAME_BYTES);
        if self.formats.contains_key(&identity)
            || self
                .names
                .get(descriptor.name())
                .is_some_and(|versions| versions.contains_key(&descriptor.version()))
        {
            return Err(Denial::DuplicateContentFormat);
        }
        let installed = RelationalContentFormat(Arc::new(InstalledFormat {
            descriptor: descriptor.clone(),
            maximum,
            codec,
        }));
        self.names
            .entry(descriptor.name().to_owned())
            .or_default()
            .insert(descriptor.version(), identity);
        self.formats.insert(identity, installed);
        self.refresh_snapshot();
        Ok(())
    }
    pub fn bind_field(
        &mut self,
        binding: CanonicalDigestId,
        format: CanonicalDigestId,
    ) -> Result<(), Denial> {
        self.get(format)?;
        if self.bindings.contains_key(&binding) {
            return Err(Denial::DuplicateContentFormat);
        }
        self.bindings.insert(binding, format);
        self.refresh_snapshot();
        Ok(())
    }
    pub fn snapshot(&self) -> super::RelationalContentSchemaSnapshot {
        self.snapshot.clone()
    }
    fn refresh_snapshot(&mut self) {
        self.snapshot = super::RelationalContentSchemaSnapshot::new(
            self.formats
                .iter()
                .map(
                    |(identity, format)| super::RelationalContentFormatDeclaration {
                        identity: *identity.bytes(),
                        descriptor: format.descriptor().clone(),
                        maximum_frame_bytes: format.maximum_frame_bytes() as u64,
                    },
                )
                .collect(),
            self.bindings
                .iter()
                .map(|(field, format)| super::RelationalContentFieldBinding {
                    field_meaning: *field.bytes(),
                    format: *format.bytes(),
                })
                .collect(),
        );
    }
    pub fn get(&self, identity: CanonicalDigestId) -> Result<&RelationalContentFormat, Denial> {
        self.formats.get(&identity).ok_or(Denial::FormatMismatch)
    }
    pub(crate) fn resolve(
        &self,
        binding: CanonicalDigestId,
    ) -> Result<&RelationalContentFormat, Denial> {
        let identity = self
            .bindings
            .get(&binding)
            .ok_or(Denial::FormatBindingMissing)?;
        self.get(*identity)
    }
    pub(crate) fn format_codec(
        &self,
        name: &str,
        version: u32,
    ) -> Result<&RelationalContentFormat, Denial> {
        let versions = self.names.get(name).ok_or(Denial::FormatMismatch)?;
        let identity = versions.get(&version).ok_or(Denial::UnsupportedVersion)?;
        self.get(*identity)
    }
    pub(super) fn lookup_bound(&self) -> u64 {
        // A B-tree node has at most11 keys; three indexed lookups compare
        // at most256 identifier bytes per key. This conservative bound is paid.
        let height = u64::from(usize::BITS - self.formats.len().leading_zeros()) + 1;
        height * 12 * 3 * 256
    }
}
