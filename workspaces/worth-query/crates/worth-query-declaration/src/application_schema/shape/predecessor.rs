use super::ApplicationAspectValueMeaning;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationAspectMeaningChange {
    Unchanged,
    RevisionOnly,
    MeaningChanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationAspectMeaningPredecessorDenial {
    DifferentAspect,
    RevisionRegressed,
    MeaningChangedWithoutRevision,
}

impl ApplicationAspectValueMeaning {
    /// Descriptive revision check, also used by golden fixtures. Success does
    /// not establish authentic predecessor or same-physical-carrier authority.
    pub fn compare_predecessor(
        &self,
        previous: &Self,
    ) -> Result<ApplicationAspectMeaningChange, ApplicationAspectMeaningPredecessorDenial> {
        use ApplicationAspectMeaningChange as Change;
        use ApplicationAspectMeaningPredecessorDenial as D;
        if self.identity() != previous.identity()
            || self.entity() != previous.entity()
            || self.aspect() != previous.aspect()
        {
            return Err(D::DifferentAspect);
        }
        if self.revision().0 < previous.revision().0 {
            return Err(D::RevisionRegressed);
        }
        if self.absence() != previous.absence() || self.fields() != previous.fields() {
            if self.revision() == previous.revision() {
                return Err(D::MeaningChangedWithoutRevision);
            }
            return Ok(Change::MeaningChanged);
        }
        Ok(if self.revision() == previous.revision() {
            Change::Unchanged
        } else {
            Change::RevisionOnly
        })
    }
}
