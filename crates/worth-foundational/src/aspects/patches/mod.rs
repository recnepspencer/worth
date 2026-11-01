mod application;
mod construction;
mod denials;
mod patch;
mod storage_bounds;
mod struct_field_application;
pub(crate) use storage_bounds::backing_bytes;
pub use storage_bounds::AspectPatchStorageBounds;

pub use denials::{AuthoritativePatchApplicationDenial, AuthoritativePatchConstructionDenial};
pub use patch::{AuthoritativeRecordAspectPatch, FieldLevelAspectPatch};
