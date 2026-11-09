//! The scalar binding owns how its value carries a Native content reference.
use super::super::{ApplicationReadableScalarValueBinding, ContentRefApplicationValueBinding};
use worth_foundational::facade::ContentRefId;

pub trait ApplicationContentReferenceBinding: ApplicationReadableScalarValueBinding {
    fn from_reference(reference: ContentRefId) -> Self::Value;
    fn reference(value: &Self::Value) -> ContentRefId;
}
impl ApplicationContentReferenceBinding for ContentRefApplicationValueBinding {
    fn from_reference(reference: ContentRefId) -> Self::Value {
        reference
    }
    fn reference(value: &Self::Value) -> ContentRefId {
        *value
    }
}
