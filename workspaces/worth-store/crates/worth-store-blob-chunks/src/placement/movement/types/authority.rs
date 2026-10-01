#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobPlacementMovementAuthority {
    _private: (),
}

impl BlobPlacementMovementAuthority {
    pub const fn for_planning() -> Self {
        Self { _private: () }
    }
}
