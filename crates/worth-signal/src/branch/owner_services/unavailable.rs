/// Stable denial posture returned when a weak service cannot enter its owner.
///
/// This is descriptive closing-or-gone vocabulary, not authority. It remains
/// serializable with inherited denial contracts; `non_exhaustive` reserves its
/// representation for compatible evolution.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignalOwnerUnavailable;
