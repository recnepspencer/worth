//! A producer's typed domain denial crosses the demand boundary as a declared
//! canonical value. The declaring binding is the sole encoder and decoder.

use crate::domain_computation::primary_graph::vocabulary::application_invariant::refusal::canonical_wire_maximum;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial,
};

mod declaration;
pub use declaration::{
    WorthQueryProducerDenialDeclaration, WorthQueryProducerDenialProjection,
    WorthQueryProjectedProducerDenial, WorthQueryUndeclaredProducerDenial,
};





