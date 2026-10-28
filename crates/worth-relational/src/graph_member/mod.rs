//! Descriptive graph coordinates, separate from view-issued member authority.
mod domain;
mod key;

pub use domain::NativeMemberDomain;
pub use key::{NativeMemberKey, ProjectedMemberCoordinate};
