//! Home-bound selection of actual World sources during Native reconstruction.
use crate::HomeIncarnationFence;
use std::sync::Arc;

#[derive(Debug)]
struct SelectionOccurrence {
    _retained: Arc<dyn std::fmt::Debug + Send + Sync>,
}

/// The actual Home owns one source admission; descriptions cannot mint it.
#[derive(Debug)]
pub struct HomeWorldSelectionOwner {
    occurrence: Arc<SelectionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
}
/// Installed with the same Home's Native medium before source admission.
#[derive(Clone, Debug)]
pub struct HomeWorldSelectionAuthority {
    occurrence: Arc<SelectionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
}
/// Immutable owner-framed source DATA. This grants no runtime service or effect.
#[derive(Clone, Debug)]
pub struct AdmittedHomeWorldSelection<Source> {
    authority: HomeWorldSelectionAuthority,
    source: Source,
}
/// Matching the actually attached receiver makes the source usable for replay.
#[derive(Debug)]
pub struct VerifiedHomeWorldSelection<Source> {
    authority: HomeWorldSelectionAuthority,
    source: Source,
}
impl<Source> VerifiedHomeWorldSelection<Source> {
    pub fn source(&self) -> &Source {
        &self.source
    }
    pub fn authority(&self) -> &HomeWorldSelectionAuthority {
        &self.authority
    }
}
impl<Source> VerifiedHomeWorldSelection<Arc<Source>> {
    /// Eligibility DATA is usable only with this exact owner-admitted source.
    pub fn require_original_source(
        &self,
        source: &Arc<Source>,
    ) -> Result<(), HomeWorldSelectionAdmissionDenial> {
        if Arc::ptr_eq(&self.source, source) {
            Ok(())
        } else {
            Err(HomeWorldSelectionAdmissionDenial::ForeignSource)
        }
    }
}
/// The Home keeps this unique continuation until actual World reconciliation.
#[derive(Debug)]
pub struct HomeWorldSelectionCompletionOwner {
    authority: HomeWorldSelectionAuthority,
}
/// Completing the same original World handoff releases only preparation pins.
#[derive(Debug)]
pub struct CompletedHomeWorldSelection {
    authority: HomeWorldSelectionAuthority,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeWorldSelectionAdmissionDenial {
    StaleIncarnation,
    ForeignOwner,
    ForeignSource,
}
impl HomeWorldSelectionOwner {
    pub fn from_retained_home_owner(
        home: [u8; 16],
        incarnation: HomeIncarnationFence,
        retained: Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Self {
        Self {
            occurrence: Arc::new(SelectionOccurrence {
                _retained: retained,
            }),
            home,
            incarnation,
        }
    }
    pub fn retained_occurrence_bytes() -> usize {
        std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<SelectionOccurrence>())
            .expect("the fixed source seal layout fits the target pointer width")
            .0
            .pad_to_align()
            .size()
    }
    pub fn authority(&self) -> HomeWorldSelectionAuthority {
        HomeWorldSelectionAuthority {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
        }
    }
    pub fn admit_source<Source>(
        self,
        source: Source,
    ) -> (
        AdmittedHomeWorldSelection<Source>,
        HomeWorldSelectionCompletionOwner,
    ) {
        let authority = self.authority();
        (
            AdmittedHomeWorldSelection {
                authority: authority.clone(),
                source,
            },
            HomeWorldSelectionCompletionOwner { authority },
        )
    }
}
impl<Source> AdmittedHomeWorldSelection<Source> {
    pub fn source(&self) -> &Source {
        &self.source
    }
}
impl HomeWorldSelectionCompletionOwner {
    pub fn complete(self) -> CompletedHomeWorldSelection {
        CompletedHomeWorldSelection {
            authority: self.authority,
        }
    }
}
impl HomeWorldSelectionAuthority {
    pub fn verify_source<Source>(
        &self,
        source: AdmittedHomeWorldSelection<Source>,
    ) -> Result<VerifiedHomeWorldSelection<Source>, HomeWorldSelectionAdmissionDenial> {
        self.require_same(&source.authority)?;
        Ok(VerifiedHomeWorldSelection {
            authority: self.clone(),
            source: source.source,
        })
    }
    pub fn admit_completion(
        &self,
        completed: &CompletedHomeWorldSelection,
    ) -> Result<(), HomeWorldSelectionAdmissionDenial> {
        self.require_same(&completed.authority)
    }
    fn require_same(&self, other: &Self) -> Result<(), HomeWorldSelectionAdmissionDenial> {
        use HomeWorldSelectionAdmissionDenial as Denial;
        if self.home == other.home && self.incarnation != other.incarnation {
            return Err(Denial::StaleIncarnation);
        }
        if self.home != other.home || !Arc::ptr_eq(&self.occurrence, &other.occurrence) {
            return Err(Denial::ForeignOwner);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
