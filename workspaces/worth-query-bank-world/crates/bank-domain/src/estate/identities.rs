macro_rules! estate_identity {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
            pub struct $name(u64);

            impl $name {
                pub const fn new(value: u64) -> Option<Self> {
                    if value == 0 {
                        None
                    } else {
                        Some(Self(value))
                    }
                }

                pub const fn get(self) -> u64 {
                    self.0
                }

                pub fn canonical_text(self) -> String {
                    format!("fixture:{}", self.0)
                }

                pub fn parse_canonical_text(value: &str) -> Option<Self> {
                    let value = value.strip_prefix("fixture:")?.parse::<u64>().ok()?;
                    Self::new(value)
                }
            }
        )+
    };
}

estate_identity!(
    BranchId,
    CapabilityGrantId,
    DeathNoticeId,
    EmergencyAccessId,
    EstateCaseId,
    LegalAuthorityId,
    MandatoryReviewId,
);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
pub struct EstateMoment(u64);

impl EstateMoment {
    pub const fn from_epoch_seconds(value: u64) -> Self {
        Self(value)
    }

    pub const fn epoch_seconds(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
pub struct DelegationLimit(u8);

impl DelegationLimit {
    pub const fn none() -> Self {
        Self(0)
    }

    pub const fn generations(value: u8) -> Self {
        Self(value)
    }

    pub const fn remaining(self) -> u8 {
        self.0
    }
}
