/// Correlates primitive attempts within one media-owner incarnation.
///
/// It is diagnostic identity, not persisted artifact identity or authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaOperationIdentity(u64);

impl MediaOperationIdentity {
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Creates a diagnostic correlation identity for an effect performed by
    /// the Store-owned recovery media boundary. This identity is descriptive
    /// and opens no backend capability.
    #[cfg(feature = "recovery-runtime-owner")]
    pub const fn from_recovery_effect(value: std::num::NonZeroU64) -> Self {
        Self(value.get())
    }

    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }

    #[cfg(test)]
    pub(super) const fn for_test(value: u64) -> Self {
        Self(value)
    }
}

/// Identifies one live filesystem-media owner incarnation.
///
/// It is ephemeral correlation, not stable Store identity or authority by
/// itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaOwnerIdentity([u8; 16]);

impl MediaOwnerIdentity {
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }

    pub(super) fn generate() -> Result<Self, getrandom::Error> {
        loop {
            let mut bytes = [0_u8; 16];
            getrandom::fill(&mut bytes)?;
            if bytes != [0_u8; 16] {
                return Ok(Self(bytes));
            }
        }
    }

    #[cfg(test)]
    pub(super) const fn for_test(value: u8) -> Self {
        Self([value; 16])
    }
}

/// Correlates diagnostics with a handle held by one exact media owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaHandleIdentity {
    owner: MediaOwnerIdentity,
    generation: u64,
}

impl MediaHandleIdentity {
    pub const fn owner(self) -> MediaOwnerIdentity {
        self.owner
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub(super) const fn new(owner: MediaOwnerIdentity, generation: u64) -> Self {
        Self { owner, generation }
    }

    #[cfg(test)]
    pub(super) const fn for_test(value: u64) -> Self {
        Self::new(MediaOwnerIdentity::for_test(1), value)
    }
}

/// Binds concrete capability handles to one qualified media environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaQualificationIdentity(u64);

impl MediaQualificationIdentity {
    pub const fn value(self) -> u64 {
        self.0
    }

    pub(super) fn generate() -> Option<Self> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        {
            let update_counter = &NEXT;
            let mut current = update_counter.load(std::sync::atomic::Ordering::Relaxed);
            loop {
                let next_value = current.checked_add(1);
                let Some(updated_value) = next_value else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .ok()
        .map(Self)
    }
}
