//! Finite carrier/case domains issued by the actual Native schema owner.
pub(crate) mod canonical;
mod catalog;
mod definition;
mod projection;

pub(crate) use definition::RegisteredProjectedMemberDomain;
pub use definition::{
    ProjectedMemberCaseHandle, ProjectedMemberDomainDenial, ProjectedMemberDomainHandle,
    ProjectedMemberDomainIdentity, ProjectedMemberDomainParts,
};
pub use projection::ObservedProjectedMember;
