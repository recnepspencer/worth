//! Schema composition retains actual frozen executable handles.
use super::{Denial, RelationalContentFormatRegistry};
impl RelationalContentFormatRegistry {
    pub(crate) fn extend(mut self, additions: Self) -> Result<Self, Denial> {
        for (identity, format) in additions.formats {
            if self.formats.contains_key(&identity)
                || self
                    .names
                    .get(format.descriptor().name())
                    .is_some_and(|versions| versions.contains_key(&format.descriptor().version()))
            {
                return Err(Denial::DuplicateContentFormat);
            }
            self.names
                .entry(format.descriptor().name().to_owned())
                .or_default()
                .insert(format.descriptor().version(), identity);
            self.formats.insert(identity, format);
        }
        for (field, identity) in additions.bindings {
            self.get(identity)?;
            if self.bindings.insert(field, identity).is_some() {
                return Err(Denial::DuplicateContentFormat);
            }
        }
        self.refresh_snapshot();
        Ok(self)
    }
}
