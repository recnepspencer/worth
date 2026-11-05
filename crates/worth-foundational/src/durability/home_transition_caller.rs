//! Portable descriptive bytes; only the attached Native owner admits execution.
use super::{
    CompositeAttemptAssociation, HomeIdentity, HomeProgramTransitionCatalogDigest,
    HomeProgramTransitionDescriptionDenial, HomeProgramTransitionIdentity,
    HomeProgramTransitionPlanDescription, HomeProgramTransitionStepAssociation,
};
use crate::boundary_protocol::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity,
    BoundaryProtocolUnsupportedVersion, BoundaryProtocolVersion,
};
use std::num::NonZeroU64;

const MAGIC: &[u8; 8] = b"WHMSTEP1";
const HEADER: usize = 113;

/// Execution meaning relative to the plan's one target-installation step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeProgramTransitionStepRole {
    BeforeTarget,
    Target,
    AfterTarget,
}

impl HomeProgramTransitionPlanDescription {
    pub fn step_role(
        &self,
        step: u32,
    ) -> Result<HomeProgramTransitionStepRole, HomeProgramTransitionDescriptionDenial> {
        if step >= self.steps() {
            return Err(HomeProgramTransitionDescriptionDenial::StepOutOfRange);
        }
        Ok(match step.cmp(&self.target_meaning().step()) {
            std::cmp::Ordering::Less => HomeProgramTransitionStepRole::BeforeTarget,
            std::cmp::Ordering::Equal => HomeProgramTransitionStepRole::Target,
            std::cmp::Ordering::Greater => HomeProgramTransitionStepRole::AfterTarget,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeProgramTransitionCallerDenial {
    Malformed,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    Description(HomeProgramTransitionDescriptionDenial),
    Length,
}

/// Borrows the original World association; grants neither admission nor custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionCaller<'bytes> {
    step: HomeProgramTransitionStepAssociation,
    role: HomeProgramTransitionStepRole,
    catalog: HomeProgramTransitionCatalogDigest,
    commitment: [u8; 32],
    original: &'bytes [u8],
}

impl<'bytes> HomeProgramTransitionCaller<'bytes> {
    pub const PROTOCOL: BoundaryProtocolIdentity =
        BoundaryProtocolIdentity::new("worth.home.transition.caller");
    pub const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);

    pub fn for_step(
        plan: &HomeProgramTransitionPlanDescription,
        step: HomeProgramTransitionStepAssociation,
        original: &CompositeAttemptAssociation,
        bytes: &'bytes [u8],
    ) -> Result<Self, HomeProgramTransitionCallerDenial> {
        use HomeProgramTransitionCallerDenial as Denial;
        if bytes.is_empty() || u64::try_from(bytes.len()).is_err() {
            return Err(Denial::Length);
        }
        let commitment = plan
            .step_commitment(step, original)
            .map_err(Denial::Description)?;
        Ok(Self {
            step,
            role: plan.step_role(step.step()).map_err(Denial::Description)?,
            catalog: plan.target_meaning().catalog(),
            commitment,
            original: bytes,
        })
    }

    /// Recognize the family even when its version or payload is unsupported.
    pub fn recognizes(bytes: &[u8]) -> bool {
        bytes.starts_with(MAGIC)
    }

    pub fn read(bytes: &'bytes [u8]) -> Result<Self, HomeProgramTransitionCallerDenial> {
        use HomeProgramTransitionCallerDenial as Denial;
        if bytes.len() < HEADER || !Self::recognizes(bytes) {
            return Err(Denial::Malformed);
        }
        let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
            bytes[8..12].try_into().expect("fixed header"),
        ))
        .map_err(|_| Denial::Malformed)?;
        BoundaryProtocolCompatibilityWindow::inclusive(Self::VERSION, Self::VERSION)
            .admit(version)
            .map_err(Denial::UnsupportedVersion)?;
        let home =
            HomeIdentity::from_observed_bytes(bytes[12..28].try_into().expect("fixed header"))
                .map_err(|_| Denial::Malformed)?;
        let intent = NonZeroU64::new(u64::from_le_bytes(
            bytes[28..36].try_into().expect("fixed header"),
        ))
        .ok_or(Denial::Malformed)?;
        let step = u32::from_le_bytes(bytes[36..40].try_into().expect("fixed header"));
        let role = match bytes[40] {
            0 => HomeProgramTransitionStepRole::BeforeTarget,
            1 => HomeProgramTransitionStepRole::Target,
            2 => HomeProgramTransitionStepRole::AfterTarget,
            _ => return Err(Denial::Malformed),
        };
        let length = u64::from_le_bytes(bytes[105..113].try_into().expect("fixed header"));
        if length == 0 || usize::try_from(length).ok() != Some(bytes.len() - HEADER) {
            return Err(Denial::Length);
        }
        Ok(Self {
            step: HomeProgramTransitionStepAssociation::from_recorded(
                HomeProgramTransitionIdentity::from_recorded(home, intent),
                step,
            ),
            role,
            catalog: HomeProgramTransitionCatalogDigest::from_recorded(
                bytes[41..73].try_into().expect("fixed header"),
            ),
            commitment: bytes[73..105].try_into().expect("fixed header"),
            original: &bytes[HEADER..],
        })
    }

    pub fn header(self) -> [u8; HEADER] {
        let mut header = [0; HEADER];
        header[..8].copy_from_slice(MAGIC);
        header[8..12].copy_from_slice(&Self::VERSION.get().to_le_bytes());
        header[12..28].copy_from_slice(&self.step.transition().home().bytes());
        header[28..36]
            .copy_from_slice(&self.step.transition().intent_position().get().to_le_bytes());
        header[36..40].copy_from_slice(&self.step.step().to_le_bytes());
        header[40] = match self.role {
            HomeProgramTransitionStepRole::BeforeTarget => 0,
            HomeProgramTransitionStepRole::Target => 1,
            HomeProgramTransitionStepRole::AfterTarget => 2,
        };
        header[41..73].copy_from_slice(&self.catalog.bytes());
        header[73..105].copy_from_slice(&self.commitment);
        header[105..113].copy_from_slice(&(self.original.len() as u64).to_le_bytes());
        header
    }

    pub const fn step(self) -> HomeProgramTransitionStepAssociation {
        self.step
    }
    pub const fn role(self) -> HomeProgramTransitionStepRole {
        self.role
    }
    pub const fn catalog(self) -> HomeProgramTransitionCatalogDigest {
        self.catalog
    }
    pub const fn commitment(self) -> [u8; 32] {
        self.commitment
    }
    pub const fn original(self) -> &'bytes [u8] {
        self.original
    }
}

#[cfg(test)]
mod tests;
