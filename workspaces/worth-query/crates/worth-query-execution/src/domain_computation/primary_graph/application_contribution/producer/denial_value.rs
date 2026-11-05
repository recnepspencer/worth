//! A producer's typed domain denial crosses the demand boundary as a declared
//! canonical value. The declaring binding is the sole encoder and decoder.

mod declaration;
pub use declaration::{
    WorthQueryProducerDenialDeclaration, WorthQueryProducerDenialProjection,
    WorthQueryProjectedProducerDenial, WorthQueryUndeclaredProducerDenial,
};
