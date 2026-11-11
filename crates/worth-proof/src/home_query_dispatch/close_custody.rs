//! Permission carried from dispatch settlement into Home retirement.
use super::AdmittedHomeQueryDispatch;

/// What close may rely on for the dispatches this runtime made.
///
/// An admitted Home prefix carries durable retirement permission.
/// ```
/// use worth_proof::{AdmittedHomeQueryDispatch, QueryDispatchCloseCustody};
/// fn retain(prefix: AdmittedHomeQueryDispatch) -> QueryDispatchCloseCustody {
///     QueryDispatchCloseCustody::Durable(prefix)
/// }
/// ```
/// A no-effect frame publication cannot substitute an admitted journal prefix.
/// ```compile_fail,E0308
/// use worth_proof::{NoEffectHomeQueryDispatch, QueryDispatchCloseCustody};
/// fn no_effect_is_not_retirement(no_effect: NoEffectHomeQueryDispatch) {
///     let _ = QueryDispatchCloseCustody::Durable(no_effect);
/// }
/// ```
#[derive(Debug)]
pub enum QueryDispatchCloseCustody {
    /// No durable journal was admitted; mandatory bases cannot retire.
    ProcessLocal,
    /// The original Home journal prefix was admitted after settlement.
    Durable(AdmittedHomeQueryDispatch),
}
