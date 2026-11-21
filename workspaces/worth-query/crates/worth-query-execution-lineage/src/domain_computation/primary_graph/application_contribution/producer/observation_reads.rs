//! Static predecessor-read meaning; this contract alone issues no read authority.
use super::super::super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

pub mod evidence;
pub use evidence::WorthQueryHistoricalProducerRead;
pub mod installation;

/// It stores no retained old basis. Only an observation's synchronous loan pins it.
#[derive(Default)]
pub struct ProducerPredecessorReadState {
    pub selected: std::sync::Mutex<Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>>,
    pub historical: std::sync::Mutex<Vec<WorthQueryHistoricalProducerRead>>,
}

impl ProducerPredecessorReadState {
    pub fn record(&self, read: WorthQueryHistoricalProducerRead) {
        self.historical
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(read);
    }

    pub fn take_historical(&self) -> Option<std::sync::Arc<[WorthQueryHistoricalProducerRead]>> {
        let mut reads = self
            .historical
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if reads.is_empty() {
            return None;
        }
        Some(std::mem::take(&mut *reads).into())
    }

    pub fn selected(
        &self,
    ) -> Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication> {
        self.selected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn record_selected(
        &self,
        prior: worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
    ) -> Result<(), WorthQueryProducerObservationReadDenial> {
        let mut previous = self
            .selected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if previous
            .as_ref()
            .is_some_and(|previous| !previous.same_origin(&prior))
        {
            return Err(WorthQueryProducerObservationReadDenial::PredecessorMismatch);
        }
        *previous = Some(prior);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorthQueryProducerObservationReadDenial {
    PredecessorBasisUnavailable,
    UndeclaredObservationRead,
    PredecessorMismatch,
    Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    Control(
        Box<
            crate::domain_computation::primary_graph::application_query::SourceQueryObservationStop,
        >,
    ),
    Installation(
        worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenial,
    ),
    Projection(crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenial),
    Decision(String),
    Native(worth_relational::facade::mvcc::RelationalPredecessorBasisDenial),
}
impl From<worth_relational::facade::mvcc::RelationalPredecessorBasisDenial>
    for WorthQueryProducerObservationReadDenial
{
    fn from(value: worth_relational::facade::mvcc::RelationalPredecessorBasisDenial) -> Self {
        use worth_relational::facade::mvcc::RelationalPredecessorBasisDenial as Native;
        match value {
            Native::PredecessorBasisUnavailable => Self::PredecessorBasisUnavailable,
            Native::PredecessorMismatch => Self::PredecessorMismatch,
            other => Self::Native(other),
        }
    }
}
impl std::fmt::Display for WorthQueryProducerObservationReadDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for WorthQueryProducerObservationReadDenial {}

pub struct WorthQueryProducerObservationReadOperation<S> {
    name: &'static str,
    input: &'static str,
    operation_type: fn() -> EqualityTypeId,
    validate: fn(
        &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
}
impl<S: ApplicationSchema> WorthQueryProducerObservationReadOperation<S> {
    /// Names a typed schema operation. Installation must still prove that its
    /// actual installed meaning is read-only; a label or Rust type cannot do so.
    pub const fn new<O, I>(operation: ApplicationOperationRef<S, O, I>) -> Self
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        Self {
            name: operation.name(),
            input: O::InputBinding::IDENTITY_NAME,
            operation_type: EqualityTypeId::of::<O>,
            validate: installation::operation::<S, O, I>,
        }
    }
}
impl<S> Copy for WorthQueryProducerObservationReadOperation<S> {}
impl<S> Clone for WorthQueryProducerObservationReadOperation<S> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct WorthQueryProducerObservationReadContract<S: 'static> {
    operations: &'static [WorthQueryProducerObservationReadOperation<S>],
}
impl<S: ApplicationSchema> WorthQueryProducerObservationReadContract<S> {
    pub const fn new(operations: &'static [WorthQueryProducerObservationReadOperation<S>]) -> Self {
        Self { operations }
    }
    /// Membership checks never authorize execution. The observation owner must
    /// separately admit the current principal and pin the genuine predecessor.
    pub fn require<O: ApplicationOperationMarkerIdentity<S> + 'static>(
        &self,
    ) -> Result<(), WorthQueryProducerObservationReadDenial> {
        self.require_identity(
            O::IDENTIFIER,
            O::InputBinding::IDENTITY_NAME,
            EqualityTypeId::of::<O>(),
        )
    }
    pub fn require_identity(
        &self,
        name: &str,
        input: &str,
        operation: EqualityTypeId,
    ) -> Result<(), WorthQueryProducerObservationReadDenial> {
        if self.operations.iter().any(|read| {
            read.name == name && read.input == input && (read.operation_type)() == operation
        }) {
            Ok(())
        } else {
            Err(WorthQueryProducerObservationReadDenial::UndeclaredObservationRead)
        }
    }
    pub fn meaning(&self) -> Vec<ProducerObservationReadMeaning> {
        let mut reads = self
            .operations
            .iter()
            .map(|r| ProducerObservationReadMeaning {
                name: r.name,
                input: r.input,
                operation_type: (r.operation_type)(),
            })
            .collect::<Vec<_>>();
        reads.sort_by_key(|r| (r.name, r.input));
        reads
    }
    /// Validate static meaning against a genuine installed schema. This issues no read grant.
    pub fn validate(
        &self,
        schema: &WorthQueryInstalledApplicationSchema<S>,
        ownership: worth_foundational::facade::ProducerPublicationOwnership,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if ownership != worth_foundational::facade::ProducerPublicationOwnership::SourceScopeCascade
        {
            return Err(refused("predecessor reads require source-scope ownership"));
        }
        let mut seen = std::collections::HashSet::new();
        for read in self.operations {
            if !seen.insert(read.name) {
                return Err(refused(read.name));
            }
            (read.validate)(schema)?;
        }
        Ok(())
    }
}
impl<S> Copy for WorthQueryProducerObservationReadContract<S> {}
impl<S> Clone for WorthQueryProducerObservationReadContract<S> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProducerObservationReadMeaning {
    pub name: &'static str,
    pub input: &'static str,
    operation_type: EqualityTypeId,
}
fn refused(subject: &str) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        Kind::ProducerObservationReadContractRejected,
        subject,
    )
}
