//! Home retires a consumed delivery through World, never through copied facts.
use super::ConsumedCompositePublication;
use crate::history::PublicationFactsRetirement;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::{
    branch::RelationalBranchBasisRetirementDenial, runtime::RelationalRuntime,
};

#[derive(Debug)]
pub enum ConsumedPublicationRetirementDenial {
    UndeliveredCustody,
    UnsettledComponent,
    Native(RelationalBranchBasisRetirementDenial),
    Execution(WorkCeilingDenial),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConsumedPublicationRetirementProgress {
    Advanced,
    Complete,
}

struct Backing {
    original: Option<ConsumedCompositePublication>,
    facts: Option<PublicationFactsRetirement>,
    complete: bool,
}
/// Vacant storage funded before Home seals World; it carries no publication.
#[must_use]
pub struct PreparedConsumedPublicationRetirement {
    backing: Box<Backing>,
    _funding: Option<ExecutionMemoryReservation>,
}
/// Owns the actual consumed publication and every partly retired original share.
#[must_use]
pub struct ConsumedPublicationRetirement {
    allocation: PreparedConsumedPublicationRetirement,
}
/// Contains no publication/read accessor; explicit Home reclaim destroys its data.
/// ```compile_fail
/// use worth_runtime_world::facade::RetiredConsumedPublication;
/// fn cannot_read(value: RetiredConsumedPublication) { value.component_results(); }
/// ```
#[must_use]
pub struct RetiredConsumedPublication {
    _allocation: PreparedConsumedPublicationRetirement,
}

impl ConsumedCompositePublication {
    fn preflight_home_retirement(&self) -> Result<(), ConsumedPublicationRetirementDenial> {
        use ConsumedPublicationRetirementDenial as Denial;
        self.delivery
            .preflight_home_retirement()
            .map_err(|()| Denial::UndeliveredCustody)?;
        self.facts()
            .component_results
            .preflight_home_retirement()
            .map_err(|()| Denial::UnsettledComponent)
    }
    pub fn prepare_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedConsumedPublicationRetirement, ConsumedPublicationRetirementDenial> {
        use ConsumedPublicationRetirementDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        self.preflight_home_retirement()?;
        let funding = request
            .reserve_retained_memory(std::mem::size_of::<Backing>() as u64)
            .map_err(Denial::Execution)?;
        Ok(PreparedConsumedPublicationRetirement {
            backing: Box::new(Backing {
                original: None,
                facts: None,
                complete: false,
            }),
            _funding: funding,
        })
    }
}
impl PreparedConsumedPublicationRetirement {
    /// Prepaid storage is not permission to retire another delivery. Refusal
    /// returns both original owners, so the caller can finish its handoffs.
    pub fn begin(
        mut self,
        publication: ConsumedCompositePublication,
    ) -> Result<
        ConsumedPublicationRetirement,
        (
            Self,
            ConsumedCompositePublication,
            ConsumedPublicationRetirementDenial,
        ),
    > {
        if let Err(cause) = publication.preflight_home_retirement() {
            return Err((self, publication, cause));
        }
        assert!(self.backing.original.is_none());
        self.backing.original = Some(publication);
        Ok(ConsumedPublicationRetirement { allocation: self })
    }
}
impl ConsumedPublicationRetirement {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<ConsumedPublicationRetirementProgress, ConsumedPublicationRetirementDenial> {
        use ConsumedPublicationRetirementDenial as Denial;
        use ConsumedPublicationRetirementProgress as Progress;
        let state = &mut self.allocation.backing;
        if state.complete {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        if state.facts.is_none() {
            let original = state.original.as_mut().expect("actual consumed delivery");
            original
                .delivery
                .preflight_home_retirement()
                .map_err(|()| Denial::UndeliveredCustody)?;
            state.facts = Some(PublicationFactsRetirement::new(
                original.delivery.take_envelope_for_retirement(),
            ));
            return Ok(Progress::Advanced);
        }
        state.complete = state
            .facts
            .as_mut()
            .expect("same original facts")
            .advance(request, native)
            .map_err(Denial::Native)?;
        Ok(if state.complete {
            Progress::Complete
        } else {
            Progress::Advanced
        })
    }
    pub fn into_retired(self) -> Result<RetiredConsumedPublication, Self> {
        if self.allocation.backing.complete {
            Ok(RetiredConsumedPublication {
                _allocation: self.allocation,
            })
        } else {
            Err(self)
        }
    }
}

impl RetiredConsumedPublication {
    /// Explicitly unbudgeted destruction after the owning Home retires.
    pub fn reclaim(self) {
        drop(self);
    }
}
