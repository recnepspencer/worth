//! Preserve native codec refusal at a one-shot owner's final handoff.
use super::*;
use crate::domain_computation::primary_graph::{
    value_decode::WorthQueryValueDecodeDenial, WorthQueryApplicationProjectionDenialKind,
};

pub(in crate::domain_computation::primary_graph::application_query) fn value_decode_denial(
    cause: WorthQueryValueDecodeDenial,
    query: &str,
) -> WorthQueryApplicationOneShotDenial {
    denial(
        WorthQueryApplicationOneShotDenialKind::Projection(
            WorthQueryApplicationProjectionDenialKind::ValueDecode(cause),
        ),
        query,
        "value-decode",
    )
}
