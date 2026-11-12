use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod admission;
mod aftermath;
mod continuity;
mod continuity_correspondence;
mod explanation;
mod invariant_capability;
mod support;
mod workflow;
mod workflow_inspection;

use super::proof_integration::{
    create_requested_domain_capability_contribution, AllowedContributionBinding,
    WorthQueryRequestedDomainCapabilityContribution,
};
use super::targets::WorthQueryDomainCapabilityTargetBinding;
pub use admission::*;
pub use aftermath::*;
pub use continuity::*;
pub use explanation::*;
pub use invariant_capability::*;
pub use support::*;
pub use workflow::*;

fn bind_requested<P, T>(
    payload: P,
    target: T,
) -> Result<WorthQueryRequestedDomainCapabilityContribution<P, T>, IdentityDerivationDenial>
where
    P: super::payloads::WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
    (P, T): AllowedContributionBinding<P, T>,
{
    create_requested_domain_capability_contribution(target, payload)
}
