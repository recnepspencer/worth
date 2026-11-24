//! Descriptive graph coordinates, separate from view-issued member authority.
pub mod domain;
pub mod key;

pub use domain::NativeMemberDomain;
pub use key::{NativeMemberKey, ProjectedMemberCoordinate, ProjectedMemberStorage};
