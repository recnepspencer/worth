mod application;
mod application_retention;
mod construction;
mod denials;
#[cfg(test)]
mod field_retention_tests;
mod patch;
#[cfg(test)]
mod retention_allocation_tests;
mod storage_bounds;
mod struct_field_application;
pub(crate) use storage_bounds::backing_bytes;
pub use storage_bounds::AspectPatchStorageBounds;

pub use denials::{AuthoritativePatchApplicationDenial, AuthoritativePatchConstructionDenial};
pub use patch::{AuthoritativeRecordAspectPatch, FieldLevelAspectPatch};
