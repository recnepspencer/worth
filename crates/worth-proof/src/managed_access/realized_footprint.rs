//! Installed, captured influence is distinct from READ and publication authority.
use super::{AdmittedManagedIndexCoverage, ManagedIndexCoverageAuthority};
use std::{any::Any, sync::Arc};

/// The actual installed Query verifier keeps this issuer and checks semantic
/// containment before completing the captured transition.
pub struct RealizedFootprintIssuer {
    owner: Arc<()>,
    coverage: ManagedIndexCoverageAuthority,
}
#[derive(Clone)]
pub struct RealizedFootprintAuthority(Arc<()>);

/// A separately created issuer cannot open an installed receiver.
/// ```compile_fail
/// let proof = worth_proof::VerifiedRealizedFootprint {
///     owner: std::sync::Arc::new(()),
/// };
/// ```
pub struct VerifiedRealizedFootprint {
    owner: Arc<()>,
}

/// Borrowed correspondence only. These inputs cannot construct a verified state.
#[derive(Clone, Copy)]
pub struct BorrowedRealizedFootprintFrame<'a> {
    admitted: &'a dyn Any,
    source: &'a dyn Any,
    parameters: &'a dyn Any,
    audience: &'a dyn Any,
}
impl<'a> BorrowedRealizedFootprintFrame<'a> {
    pub fn new(
        admitted: &'a dyn Any,
        source: &'a dyn Any,
        parameters: &'a dyn Any,
        audience: &'a dyn Any,
    ) -> Self {
        Self {
            admitted,
            source,
            parameters,
            audience,
        }
    }
    fn same_invocation(&self, expected: &Self) -> bool {
        std::ptr::eq(self.admitted, expected.admitted)
            && std::ptr::eq(self.source, expected.source)
            && std::ptr::eq(self.parameters, expected.parameters)
            && std::ptr::eq(self.audience, expected.audience)
    }
}

pub struct CapturedRealizedFootprint<'a> {
    issuer: &'a RealizedFootprintIssuer,
    frame: BorrowedRealizedFootprintFrame<'a>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealizedFootprintDenial {
    ForeignCoverage,
    ForeignInvocation,
}
impl RealizedFootprintIssuer {
    pub fn fresh(coverage: ManagedIndexCoverageAuthority) -> Self {
        Self {
            owner: Arc::new(()),
            coverage,
        }
    }
    pub fn authority(&self) -> RealizedFootprintAuthority {
        RealizedFootprintAuthority(Arc::clone(&self.owner))
    }
    /// Query supplies actual verified coverage after checking installed scope
    /// containment and retaining all influencing bodies, including exact rejects.
    pub fn capture<'a>(
        &'a self,
        frame: BorrowedRealizedFootprintFrame<'a>,
        coverage: &AdmittedManagedIndexCoverage,
    ) -> Result<CapturedRealizedFootprint<'a>, RealizedFootprintDenial> {
        if !self.coverage.admits(coverage) {
            return Err(RealizedFootprintDenial::ForeignCoverage);
        }
        Ok(CapturedRealizedFootprint {
            issuer: self,
            frame,
        })
    }
}
impl CapturedRealizedFootprint<'_> {
    pub fn verify(
        self,
        expected: BorrowedRealizedFootprintFrame<'_>,
    ) -> Result<VerifiedRealizedFootprint, RealizedFootprintDenial> {
        if !self.frame.same_invocation(&expected) {
            return Err(RealizedFootprintDenial::ForeignInvocation);
        }
        Ok(VerifiedRealizedFootprint {
            owner: Arc::clone(&self.issuer.owner),
        })
    }
}
impl RealizedFootprintAuthority {
    pub fn admits(&self, proof: &VerifiedRealizedFootprint) -> bool {
        Arc::ptr_eq(&self.0, &proof.owner)
    }
}

#[cfg(test)]
mod tests;
