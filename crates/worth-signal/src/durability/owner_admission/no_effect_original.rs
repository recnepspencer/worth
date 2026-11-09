//! Retry evidence retains the actual live admission registry and original frame.
use super::*;
use worth_foundational::facade::SignalDurableJournal;
use worth_proof::ProvenNoEffectPublication;

/// Runtime custody behind the concrete no-effect retry proof. Recorded owner
/// labels, frame descriptions and unrelated authority markers cannot create it.
pub struct SignalNoEffectOriginal {
    state: Arc<AdmissionState>,
    holder: SignalLiveOwnerIdentity,
    journal: SignalDurableJournal,
    digest: [u8; 32],
}
impl worth_proof::AuthorityMarker for SignalNoEffectOriginal {}
impl std::fmt::Debug for SignalNoEffectOriginal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalNoEffectOriginal")
            .field("holder", &self.holder)
            .field("journal", &self.journal)
            .field("digest", &self.digest)
            .finish()
    }
}
impl SignalDurableOwnerClaim {
    pub(crate) fn no_effect_original(
        &self,
        journal: SignalDurableJournal,
        digest: [u8; 32],
    ) -> ProvenNoEffectPublication<SignalNoEffectOriginal> {
        ProvenNoEffectPublication::from_owner(SignalNoEffectOriginal {
            state: Arc::clone(&self.state),
            holder: self.identity,
            journal,
            digest,
        })
    }
}
impl SignalNoEffectOriginal {
    /// A medium must check its actual registry and exact subject before retry.
    /// It must independently retain a proven-no-effect physical endpoint.
    pub fn admit_retry(
        &self,
        registry: &SignalDurableOwnerAdmissionRegistry,
        journal: SignalDurableJournal,
        digest: [u8; 32],
    ) -> Result<(), D> {
        let posture = registry.posture()?;
        let observed = match *posture {
            AdmissionPosture::Claimed(holder) => Some(holder),
            AdmissionPosture::Available
            | AdmissionPosture::RetiredClaimed(_)
            | AdmissionPosture::Retired => None,
        };
        if !Arc::ptr_eq(&self.state, &registry.state) || observed != Some(self.holder) {
            return Err(D::PublicationRetryOwnerMismatch {
                expected: self.holder,
                observed,
            });
        }
        if self.journal != journal || self.digest != digest {
            return Err(D::PublicationRetrySubjectMismatch {
                expected_journal: self.journal,
                expected_digest: self.digest,
                observed_journal: journal,
                observed_digest: digest,
            });
        }
        Ok(())
    }
}
