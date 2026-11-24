//! Runtime lifecycle delegation for the physical storage owners.
use super::{RecordIdentitySubsystem, RuntimeSubsystem, StorageSubsystem};

impl RuntimeSubsystem for RecordIdentitySubsystem {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self::default()
    }

    fn fork(&self) -> Self {
        self.detached()
    }
}

impl RuntimeSubsystem for StorageSubsystem {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self::default()
    }

    fn fork(&self) -> Self {
        self.detached()
    }
}
