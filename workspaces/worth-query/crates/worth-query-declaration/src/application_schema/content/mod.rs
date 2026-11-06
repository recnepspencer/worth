//! Portable content meaning and typed field bindings; Native owns live bytes.
mod binding;
mod field;
mod recipe;
pub(super) mod validation;
pub use binding::{
    ApplicationContentBinding, ApplicationContentCodecDenial, ApplicationContentControl,
    ApplicationContentEncoder, ApplicationContentFormatDeclaration,
};
pub use field::{content_field, ApplicationContentFieldRef};
pub use recipe::{
    ApplicationContentBindingRecipe, ErasedContentEncode, ErasedContentLength,
    ErasedContentValidate, ErasedContentVerify,
};

mod reference_binding;
pub use reference_binding::ApplicationContentReferenceBinding;
