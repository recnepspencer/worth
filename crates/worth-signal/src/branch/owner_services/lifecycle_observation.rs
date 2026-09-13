/// Public observation of the Signal owner root's lifecycle progression.
///
/// This is descriptive state, not authority to advance or close the owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalOwnerLifecycleObservation {
    Open,
    Closing,
    Closed,
}
