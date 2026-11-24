//! Finite carrier/case domains issued by the actual Native schema owner.
pub mod canonical;
pub mod catalog;
pub mod definition;

pub use definition::RegisteredProjectedMemberDomain;
pub use definition::{
    ProjectedMemberCaseHandle, ProjectedMemberDomainDenial, ProjectedMemberDomainHandle,
    ProjectedMemberDomainIdentity, ProjectedMemberDomainParts,
};
