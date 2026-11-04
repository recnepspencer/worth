//! Sealed test doors over this module's private members.
use super::*;

impl PreparedReadyBacking {
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        Self::new()
    }
}

impl PreparedReadyBacking {
    #[doc(hidden)]
    pub fn for_registry_test_for_test() -> Self {
        Self::for_registry_test()
    }
}

impl PreparedReadyBacking {
    #[doc(hidden)]
    pub fn for_test_for_test() -> Self {
        Self::for_test()
    }
}

impl PreparedReadyBacking {
    #[doc(hidden)]
    pub fn generation_for_test(&self) -> OutputCheckpointGeneration {
        Self::generation(self)
    }
}
