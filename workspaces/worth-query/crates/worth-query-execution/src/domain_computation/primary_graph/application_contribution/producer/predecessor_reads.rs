//! Static predecessor-read meaning; this contract alone issues no read authority.
use super::super::super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

mod installation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorthQueryPredecessorReadDenial {
    PredecessorBasisUnavailable,
    UndeclaredPredecessorRead,
    PredecessorMismatch,
    Native(worth_relational::facade::mvcc::RelationalPredecessorBasisDenial),
}
impl From<worth_relational::facade::mvcc::RelationalPredecessorBasisDenial>
    for WorthQueryPredecessorReadDenial
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
impl std::fmt::Display for WorthQueryPredecessorReadDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for WorthQueryPredecessorReadDenial {}

pub struct WorthQueryPredecessorReadOperation<S> {
    name: &'static str,
    input: &'static str,
    operation_type: fn() -> EqualityTypeId,
    validate: fn(
        &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
}
impl<S: ApplicationSchema> WorthQueryPredecessorReadOperation<S> {
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
impl<S> Copy for WorthQueryPredecessorReadOperation<S> {}
impl<S> Clone for WorthQueryPredecessorReadOperation<S> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct WorthQueryPredecessorReadContract<S: 'static> {
    operations: &'static [WorthQueryPredecessorReadOperation<S>],
}
impl<S: ApplicationSchema> WorthQueryPredecessorReadContract<S> {
    pub const fn new(operations: &'static [WorthQueryPredecessorReadOperation<S>]) -> Self {
        Self { operations }
    }
    /// Membership checks never authorize execution. The observation owner must
    /// separately admit the current principal and pin the genuine predecessor.
    pub fn require<O: ApplicationOperationMarkerIdentity<S> + 'static>(
        &self,
    ) -> Result<(), WorthQueryPredecessorReadDenial> {
        if self.operations.iter().any(|read| {
            read.name == O::IDENTIFIER
                && read.input == O::InputBinding::IDENTITY_NAME
                && (read.operation_type)() == EqualityTypeId::of::<O>()
        }) {
            Ok(())
        } else {
            Err(WorthQueryPredecessorReadDenial::UndeclaredPredecessorRead)
        }
    }
    pub(super) fn meaning(&self) -> Vec<PredecessorReadMeaning> {
        let mut reads = self
            .operations
            .iter()
            .map(|r| PredecessorReadMeaning {
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
        ownership: super::ProducerPublicationOwnership,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if ownership != super::ProducerPublicationOwnership::SourceScopeCascade {
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
impl<S> Copy for WorthQueryPredecessorReadContract<S> {}
impl<S> Clone for WorthQueryPredecessorReadContract<S> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::domain_computation::primary_graph) struct PredecessorReadMeaning {
    pub(super) name: &'static str,
    pub(super) input: &'static str,
    operation_type: EqualityTypeId,
}
fn refused(subject: &str) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        Kind::ProducerPredecessorReadContractRejected,
        subject,
    )
}
