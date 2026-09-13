use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum RootArtifactRole {
    CurrentSelector,
    PreviousSelector,
    AddressedRootManifest,
}
