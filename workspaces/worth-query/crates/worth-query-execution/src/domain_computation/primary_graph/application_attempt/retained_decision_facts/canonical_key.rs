use super::StoreDenial;
use std::cmp::Ordering;
use worth_execution::{ExecutionByteBuffer, ExecutionImmutableBytes};

/// Mirrors CURRENT StorageKey ordering: locator, optional predicate, numeric limit.
/// A flattened decimal/string tuple would change ordering (e.g. limit 2 vs 10).
pub(in crate::domain_computation::primary_graph) struct AdmittedFactKey {
    locator: ExecutionImmutableBytes,
    predicate: Option<(ExecutionImmutableBytes, usize)>,
}

impl AdmittedFactKey {
    /// Emit borrowed canonical data directly. No external/owned String adoption.
    /// Caller must use the canonical grammar; literal emission in tests is only
    /// a storage-mechanics oracle, NOT evidence of Query canonical encoding.
    pub(in crate::domain_computation::primary_graph) fn write(
        locator_length: usize,
        predicate: Option<(usize, usize)>,
        policy: super::StorageControl<'_, '_>,
        locator: impl FnOnce(&mut ExecutionByteBuffer) -> Result<(), StoreDenial>,
        predicate_writer: impl FnOnce(&mut ExecutionByteBuffer) -> Result<(), StoreDenial>,
    ) -> Result<Self, StoreDenial> {
        policy.check_live()?;
        let mut output = ExecutionByteBuffer::allocate(locator_length, policy.policy())?;
        locator(&mut output)?;
        let locator = output.seal()?;
        let predicate = match predicate {
            Some((length, limit)) => {
                policy.check_live()?;
                let mut output = ExecutionByteBuffer::allocate(length, policy.policy())?;
                predicate_writer(&mut output)?;
                Some((output.seal()?, limit))
            }
            None => None,
        };
        Ok(Self { locator, predicate })
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> &[u8] {
        self.locator.bytes()
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn predicate_material(
        &self,
    ) -> Option<(&[u8], usize)> {
        self.predicate
            .as_ref()
            .map(|(bytes, limit)| (bytes.bytes(), *limit))
    }
    pub(in crate::domain_computation::primary_graph) fn compare(
        &self,
        other: &Self,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Ordering, StoreDenial> {
        let locator = compare_bytes(self.locator.bytes(), other.locator.bytes(), policy)?;
        if !locator.is_eq() {
            return Ok(locator);
        }
        Ok(match (&self.predicate, &other.predicate) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some((left, limit)), Some((right, other_limit))) => {
                compare_bytes(left.bytes(), right.bytes(), policy)?
                    .then_with(|| limit.cmp(other_limit))
            }
        })
    }
}

fn compare_bytes(
    left: &[u8],
    right: &[u8],
    policy: super::StorageControl<'_, '_>,
) -> Result<Ordering, StoreDenial> {
    // Scheduling granule only. Compare every byte; do not compare hashes.
    let common = left.len().min(right.len());
    for offset in (0..common).step_by(64 * 1024) {
        policy.check_live()?;
        let end = offset.saturating_add(64 * 1024).min(common);
        let order = left[offset..end].cmp(&right[offset..end]);
        if !order.is_eq() {
            return Ok(order);
        }
    }
    policy.check_live()?;
    Ok(left.len().cmp(&right.len()))
}

impl Ord for AdmittedFactKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.locator
            .bytes()
            .cmp(other.locator.bytes())
            .then_with(|| match (&self.predicate, &other.predicate) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some((left, left_limit)), Some((right, right_limit))) => left
                    .bytes()
                    .cmp(right.bytes())
                    .then_with(|| left_limit.cmp(right_limit)),
            })
    }
}
impl PartialOrd for AdmittedFactKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for AdmittedFactKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for AdmittedFactKey {}
