#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadAccessAdmissionPosture {
    InlineIndexed,
    BoundedEphemeralIndex,
    PersistentIndexRequired,
    AsyncMaterializationRequired,
    StoreBackedCapabilityRequired,
    AccessCapabilityRegistrationRequired,
    Denied,
}

impl WorthQueryGraphReadAccessAdmissionPosture {
    pub const ALL: [Self; 7] = [
        Self::InlineIndexed,
        Self::BoundedEphemeralIndex,
        Self::PersistentIndexRequired,
        Self::AsyncMaterializationRequired,
        Self::StoreBackedCapabilityRequired,
        Self::AccessCapabilityRegistrationRequired,
        Self::Denied,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InlineIndexed => "inline_indexed",
            Self::BoundedEphemeralIndex => "bounded_ephemeral_index",
            Self::PersistentIndexRequired => "persistent_index_required",
            Self::AsyncMaterializationRequired => "async_materialization_required",
            Self::StoreBackedCapabilityRequired => "store_backed_capability_required",
            Self::AccessCapabilityRegistrationRequired => "access_capability_registration_required",
            Self::Denied => "denied",
        }
    }

    pub fn digest_part(&self) -> String {
        format!("posture:{}", self.as_str())
    }
}
