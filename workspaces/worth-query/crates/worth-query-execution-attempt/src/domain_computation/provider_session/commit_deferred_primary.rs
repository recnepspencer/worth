//! Primary-graph typed adapters for runtime session commit deferral and attempt binding.

use worth_query_execution_runtime::facade::ports::{
    OpaquePrerequisiteDenial, OpaquePrerequisiteDenialKind,
};
use worth_query_execution_runtime::facade::provider_session::{
    WorthQueryProviderSessionCommitDeferred, WorthQuerySessionBoundReadsAndEffects,
};

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptAffinity, WorthQueryApplicationAttemptBasis,
};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

pub trait CommitDeferredPrimaryExt: Sized {
    fn required_prerequisite(
        denial: WorthQueryOutputDemandDenial,
        detail: impl Into<String>,
    ) -> Self;

    fn into_prerequisite_denial(self) -> Option<WorthQueryOutputDemandDenial>;
}

impl CommitDeferredPrimaryExt for WorthQueryProviderSessionCommitDeferred {
    fn required_prerequisite(
        denial: WorthQueryOutputDemandDenial,
        detail: impl Into<String>,
    ) -> Self {
        Self::required_prerequisite_opaque(
            OpaquePrerequisiteDenialKind::new(denial.kind()),
            OpaquePrerequisiteDenial::new(denial),
            detail,
        )
    }

    fn into_prerequisite_denial(self) -> Option<WorthQueryOutputDemandDenial> {
        self.into_prerequisite_denial_opaque().map(|opaque| {
            *opaque
                .downcast::<WorthQueryOutputDemandDenial>()
                .expect("provider session prerequisite denial is WorthQueryOutputDemandDenial")
        })
    }
}

pub trait SessionBoundReadsPrimaryExt {
    fn bind_application_attempt(
        &self,
        basis: WorthQueryApplicationAttemptBasis,
    ) -> Result<WorthQueryApplicationAttemptAffinity, ()>;
}

impl SessionBoundReadsPrimaryExt for WorthQuerySessionBoundReadsAndEffects<'_> {
    fn bind_application_attempt(
        &self,
        basis: WorthQueryApplicationAttemptBasis,
    ) -> Result<WorthQueryApplicationAttemptAffinity, ()> {
        basis.bind_live_session(self.live_affinity())
    }
}

impl worth_query_execution_runtime::facade::ports::SessionBoundAttemptBasisPort
    for WorthQueryApplicationAttemptBasis
{
    type Affinity = WorthQueryApplicationAttemptAffinity;

    fn bind_to_session_bound_reads(
        self,
        reads: &WorthQuerySessionBoundReadsAndEffects<'_>,
    ) -> Result<Self::Affinity, ()> {
        self.bind_live_session(reads.live_affinity())
    }
}
