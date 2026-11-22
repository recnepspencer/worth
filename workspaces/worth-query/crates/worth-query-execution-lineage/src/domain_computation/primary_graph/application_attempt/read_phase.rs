/// Sealed phase reached by an ordinary root-scoped application read.
///
/// The marker is public only because it appears in the read-set type. Its
/// private field prevents consumers from constructing phase evidence.
pub struct WorthQueryOrdinaryApplicationRead {
    _private: (),
}

/// Sealed phase reached only after consuming an exact admission-bound
/// invariant projection.
pub struct WorthQueryProjectedApplicationMutation {
    _private: (),
}
