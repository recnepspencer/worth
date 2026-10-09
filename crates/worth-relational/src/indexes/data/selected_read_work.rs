/// Admission classification for selected reads. Payload, validation and each
/// candidate or row visit spend the caller's logical-work allowance. Only a
/// keyed ordered-container descent reports its height as navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedIndexReadWork {
    /// Budget these logical visits and payload bytes against the request allowance.
    Operation(u64),
    /// One keyed descent: charge one logical operation and bound its navigation
    /// by `MAXIMUM_ORDERED_DESCENT_WORK`. Iteration is never navigation.
    OrderedNavigation(u64),
}
impl SelectedIndexReadWork {
    /// Every ordered container has this declared representational population ceiling.
    /// A map cannot address more entries than its usize length.
    pub const MAXIMUM_ORDERED_CONTAINER_ENTRIES: usize = usize::MAX;

    /// All installed capacities fit the representational ceiling. Std B-trees compare at most eleven
    /// keys per binary-bounded level; im deletion's bound is eighteen per level.
    /// Thus this covers every descent at the largest addressable capacity.
    pub const MAXIMUM_ORDERED_DESCENT_WORK: u64 = 18 * usize::BITS as u64 + 1;
}
