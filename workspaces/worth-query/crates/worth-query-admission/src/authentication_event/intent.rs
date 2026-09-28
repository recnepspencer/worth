use std::num::NonZeroU32;
use std::time::Duration;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;

const MAX_PURPOSE_BYTES: usize = 256;

/// Exact signing meaning supplied by the approval owner, not an authority token.
/// The approval owner must derive both digests from its canonical proposal and
/// subject/dependency coverage before comparing them at the signing handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationEventIntent {
    purpose: String,
    signing_intent: CanonicalDigestId,
    subject_coverage: CanonicalDigestId,
}

impl WorthQueryAuthenticationEventIntent {
    pub fn new(
        purpose: impl Into<String>,
        signing_intent: CanonicalDigestId,
        subject_coverage: CanonicalDigestId,
    ) -> Result<Self, WorthQueryAuthenticationEventPolicyDenial> {
        let purpose = purpose.into();
        if purpose.is_empty()
            || purpose.trim() != purpose
            || purpose.len() > MAX_PURPOSE_BYTES
            || purpose.chars().any(char::is_control)
        {
            return Err(WorthQueryAuthenticationEventPolicyDenial::InvalidPurpose);
        }
        Ok(Self {
            purpose,
            signing_intent,
            subject_coverage,
        })
    }

    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    pub const fn signing_intent(&self) -> &CanonicalDigestId {
        &self.signing_intent
    }

    pub const fn subject_coverage(&self) -> &CanonicalDigestId {
        &self.subject_coverage
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthenticationEventReuse {
    SingleUse,
    AtMost(NonZeroU32),
}

impl WorthQueryAuthenticationEventReuse {
    pub(crate) const fn maximum_uses(self) -> u32 {
        match self {
            Self::SingleUse => 1,
            Self::AtMost(count) => count.get(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationEventPolicy {
    maximum_age: Duration,
    reuse: WorthQueryAuthenticationEventReuse,
}

impl WorthQueryAuthenticationEventPolicy {
    pub fn new(
        maximum_age: Duration,
        reuse: WorthQueryAuthenticationEventReuse,
    ) -> Result<Self, WorthQueryAuthenticationEventPolicyDenial> {
        if maximum_age.is_zero() || maximum_age.as_nanos() > u64::MAX as u128 {
            return Err(WorthQueryAuthenticationEventPolicyDenial::InvalidMaximumAge);
        }
        Ok(Self { maximum_age, reuse })
    }

    pub const fn maximum_age(self) -> Duration {
        self.maximum_age
    }

    pub const fn reuse(self) -> WorthQueryAuthenticationEventReuse {
        self.reuse
    }

    pub(crate) fn maximum_age_nanoseconds(self) -> u64 {
        self.maximum_age.as_nanos() as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthenticationEventPolicyDenial {
    InvalidPurpose,
    InvalidMaximumAge,
}

/// The owner, not the caller, creates this challenge. An installed verifier
/// must validate a credential against its nonce and all of its exact meaning.
///
/// ```compile_fail
/// use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventChallenge;
/// let challenge = WorthQueryAuthenticationEventChallenge::issued(
///     [0; 32], todo!(), todo!()
/// );
/// ```
pub struct WorthQueryAuthenticationEventChallenge {
    nonce: [u8; 32],
    principal: WorthQueryExternalPrincipalIdentity,
    intent: WorthQueryAuthenticationEventIntent,
}

impl WorthQueryAuthenticationEventChallenge {
    pub(crate) fn issued(
        nonce: [u8; 32],
        principal: WorthQueryExternalPrincipalIdentity,
        intent: WorthQueryAuthenticationEventIntent,
    ) -> Self {
        Self {
            nonce,
            principal,
            intent,
        }
    }

    pub const fn nonce(&self) -> &[u8; 32] {
        &self.nonce
    }

    pub const fn principal(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.principal
    }

    pub const fn intent(&self) -> &WorthQueryAuthenticationEventIntent {
        &self.intent
    }
}
