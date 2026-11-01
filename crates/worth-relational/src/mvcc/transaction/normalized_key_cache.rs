//! Sorted owned normalization meanings; replacement is one linear merge.
use crate::symbols::data::Symbol;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct NormalizedKeyCache {
    rows: Box<[(String, Symbol)]>,
}
impl NormalizedKeyCache {
    pub(super) fn get(&self, key: &str) -> Option<&Symbol> {
        self.rows
            .binary_search_by(|(name, _)| name.as_str().cmp(key))
            .ok()
            .map(|index| &self.rows[index].1)
    }
    pub(super) fn copy_bytes(&self) -> Option<usize> {
        self.rows.iter().try_fold(
            super::normalized_plan::slots::<(String, Symbol)>(self.rows.len())?,
            |bytes, (key, _)| bytes.checked_add(key.len()),
        )
    }
    pub(super) fn copy(&self) -> Self {
        Self {
            rows: self.rows.clone(),
        }
    }
    pub(super) fn replacement_bytes(
        &self,
        keys: &std::collections::BTreeSet<String>,
    ) -> Option<usize> {
        let count = self.rows.len().checked_add(keys.len())?;
        let pairs = super::normalized_plan::slots::<(String, Symbol)>(count)?;
        let nested = keys.iter().try_fold(self.copy_bytes()?, |bytes, key| {
            bytes.checked_add(key.len())
        })?;
        // Incoming rows, maximum output, and boxing shrink overlap. Existing
        // rows remain owned by the old cache and have their original bill.
        nested
            .checked_add(super::normalized_plan::slots::<(String, Symbol)>(
                keys.len(),
            )?)?
            .checked_add(pairs)?
            .checked_add(pairs)
    }
    pub(super) fn replaced(&self, mut incoming: Vec<(String, Symbol)>) -> Self {
        incoming.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let mut rows = Vec::with_capacity(
            self.rows
                .len()
                .checked_add(incoming.len())
                .expect("replacement capacity was checked before constructing incoming rows"),
        );
        let mut old = self.rows.iter().peekable();
        for next in incoming {
            while old.peek().is_some_and(|row| row.0 < next.0) {
                rows.push(old.next().expect("peeked original cache row").clone());
            }
            if old.peek().is_some_and(|row| row.0 == next.0) {
                old.next();
            }
            rows.push(next);
        }
        rows.extend(old.cloned());
        Self {
            rows: rows.into_boxed_slice(),
        }
    }
}
