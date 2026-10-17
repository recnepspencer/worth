//! Domain admission gates the unchanged declared scalar decoder.
use crate::domain_computation::primary_graph::{HandlerExecutionDenial, HandlerInterruption};
use worth_foundational::facade::AspectValue;

#[allow(clippy::type_complexity)]
pub(super) fn decode_admitted<Value, Denied>(
    raw: &AspectValue,
    admit: impl for<'raw, 'checkpoint> FnOnce(
        &'raw AspectValue,
        &'checkpoint dyn Fn() -> Result<(), HandlerInterruption>,
    ) -> Result<(), Denied>,
    checkpoint: &dyn Fn() -> Result<(), HandlerInterruption>,
    decode: impl FnOnce(&AspectValue) -> Option<Value>,
) -> Result<Result<Option<Value>, Denied>, HandlerExecutionDenial> {
    checkpoint().map_err(HandlerExecutionDenial::new)?;
    if let Err(denied) = admit(raw, checkpoint) {
        return Ok(Err(denied));
    }
    checkpoint().map_err(HandlerExecutionDenial::new)?;
    Ok(Ok(decode(raw)))
}

#[cfg(test)]
#[path = "decoding/tests.rs"]
mod tests;
