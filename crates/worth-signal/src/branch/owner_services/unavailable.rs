/// Stable denial posture returned when a weak service cannot enter its owner.
///
/// This is descriptive closing-or-gone vocabulary, not authority. It remains
/// serializable with inherited denial contracts; `non_exhaustive` reserves its
/// representation for compatible evolution.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignalOwnerUnavailable;

impl std::fmt::Display for SignalOwnerUnavailable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("The Signal owner is unavailable.")
    }
}

impl std::error::Error for SignalOwnerUnavailable {}
