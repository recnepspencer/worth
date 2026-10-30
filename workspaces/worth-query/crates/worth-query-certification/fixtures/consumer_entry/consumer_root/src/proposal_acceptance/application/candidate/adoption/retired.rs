//! Retired previous-head custody settles its original cleanup while keeping adoption evidence.
use super::*;
enum Cleanup {
    Pending(WorthQueryApplicationProductBranchCleanup),
    Refused(WorthQueryApplicationProductBranchCleanupFailure),
}
pub struct RetiredAdoption<'host> {
    evidence: AdoptionEvidence,
    host: &'host Host,
    cleanup: Cleanup,
    retirement_handoff: Option<WorthQueryAdvancementDenial>,
    cleanup_handoff: Option<WorthQueryAdvancementDenial>,
}
pub struct SettledAdoption {
    evidence: AdoptionEvidence,
    receipt: WorthQueryProductBranchOwnerCleanupReceipt,
    retirement_handoff: Option<WorthQueryAdvancementDenial>,
    cleanup_handoff: Option<WorthQueryAdvancementDenial>,
}
impl<'host> RetiredAdoption<'host> {
    pub(super) fn from_retired(
        evidence: AdoptionEvidence,
        host: &'host Host,
        cleanup: WorthQueryApplicationProductBranchCleanup,
        retirement_handoff: Option<WorthQueryAdvancementDenial>,
    ) -> Self {
        Self {
            evidence,
            host,
            cleanup: Cleanup::Pending(cleanup),
            retirement_handoff,
            cleanup_handoff: None,
        }
    }
    pub fn metadata(&self) -> &AdoptionMetadata {
        &self.evidence.metadata
    }
    pub fn completed(&self) -> &[retention_action::Completion] {
        &self.evidence.completed
    }
    pub fn cleanup_denial(&self) -> Option<&WorthQueryApplicationProductBranchCleanupDenial> {
        match &self.cleanup {
            Cleanup::Refused(failure) => Some(failure.denial()),
            Cleanup::Pending(_) => None,
        }
    }
    pub fn retirement_handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        self.retirement_handoff
    }
    pub fn cleanup_handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        self.cleanup_handoff
    }
    pub fn retry(self, caller: &Caller<'_>) -> Result<SettledAdoption, Self> {
        let Self {
            evidence,
            host,
            cleanup,
            retirement_handoff,
            ..
        } = self;
        let mut owner = Some(match cleanup {
            Cleanup::Pending(owner) => owner,
            Cleanup::Refused(failure) => failure.into_cleanup(),
        });
        let mut result = None;
        let advancement = host.with_application_advancement(caller.scope, |_| {
            result = Some(owner.take().expect("original previous cleanup").retry());
        });
        match (result, advancement) {
            (Some(Ok(receipt)), handoff) => Ok(SettledAdoption {
                evidence,
                receipt,
                retirement_handoff,
                cleanup_handoff: handoff.err(),
            }),
            (Some(Err(failure)), handoff) => Err(Self {
                evidence,
                host,
                cleanup: Cleanup::Refused(failure),
                retirement_handoff,
                cleanup_handoff: handoff.err(),
            }),
            (None, Err(cause)) => Err(Self {
                evidence,
                host,
                cleanup: Cleanup::Pending(owner.expect("callback did not consume cleanup")),
                retirement_handoff,
                cleanup_handoff: Some(cause),
            }),
            (None, Ok(())) => unreachable!("successful advancement invoked cleanup"),
        }
    }
}
impl SettledAdoption {
    pub fn metadata(&self) -> &AdoptionMetadata {
        &self.evidence.metadata
    }
    pub fn completed(&self) -> &[retention_action::Completion] {
        &self.evidence.completed
    }
    pub fn adoption_handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        self.evidence.handoff
    }
    pub fn cleanup_receipt(&self) -> &WorthQueryProductBranchOwnerCleanupReceipt {
        &self.receipt
    }
    pub fn retirement_handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        self.retirement_handoff
    }
    pub fn cleanup_handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        self.cleanup_handoff
    }
}
