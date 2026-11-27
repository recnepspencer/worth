//! Real encoded discriminator lengths and derived wire bounds as a
//! conformance trace for the proved Lean model (`proofs/lean/
//! persistent_naming`, `WireBound`).
//!
//! Each value of the modelled algebra (`bool`, `u64`, a name reference,
//! text, options and pairs of them) records its type, its value, the bytes
//! the encoder writes for it and its type's `MAX_ENCODED_BYTES`. Each whole
//! discriminator records its identity length, field names and types, field
//! values, canonical byte length and `max_encoded_bytes` bound, its
//! `MAX_BODY_BYTES` written as `#[derive(ApplicationNameDiscriminator)]`
//! writes it. With `WORTH_PERSISTENT_NAMING_TRACE_DIR=<dir>` the lines are
//! written to `<dir>/wire_bound.ndjson`, which `lake exe conformance`
//! checks against the proved `encLen`, `bound`, `discLen` and `fieldsBound`.
use std::fmt::Write as _;

use super::super::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorValue as Value,
};
use super::{bound_sum, field_bound, max_encoded_bytes};
use crate::application_naming::WorthQueryName;

/// A value of the modelled algebra with its model type and value as JSON.
trait Modelled: Value {
    fn ty() -> String;
    fn val(&self) -> String;
}

impl Modelled for bool {
    fn ty() -> String {
        r#""bool""#.into()
    }
    fn val(&self) -> String {
        format!(r#"{{"bool":{self}}}"#)
    }
}

impl Modelled for u64 {
    fn ty() -> String {
        r#""u64""#.into()
    }
    fn val(&self) -> String {
        format!(r#"{{"u64":{self}}}"#)
    }
}

impl Modelled for String {
    fn ty() -> String {
        r#""text""#.into()
    }
    fn val(&self) -> String {
        format!(r#"{{"text":{}}}"#, self.len())
    }
}

impl<K> Modelled for WorthQueryName<K> {
    fn ty() -> String {
        r#""name""#.into()
    }
    fn val(&self) -> String {
        r#""name""#.into()
    }
}

impl<T: Modelled> Modelled for Option<T> {
    fn ty() -> String {
        format!(r#"{{"opt":{}}}"#, T::ty())
    }
    fn val(&self) -> String {
        self.as_ref().map_or(r#""none""#.into(), |inner| {
            format!(r#"{{"some":{}}}"#, inner.val())
        })
    }
}

impl<A: Modelled, B: Modelled> Modelled for (A, B) {
    fn ty() -> String {
        format!(r#"{{"pair":[{},{}]}}"#, A::ty(), B::ty())
    }
    fn val(&self) -> String {
        format!(r#"{{"pair":[{},{}]}}"#, self.0.val(), self.1.val())
    }
}

fn json(bound: Option<usize>) -> String {
    bound.map_or("null".into(), |bound| bound.to_string())
}

fn value_line<T: Modelled>(value: T) -> String {
    let empty = ApplicationNameDiscriminatorEncoder::new("t").finish().len();
    let mut encoder = ApplicationNameDiscriminatorEncoder::new("t");
    value.encode_discriminator_value(&mut encoder);
    let encoded = encoder.finish().len() - empty;
    format!(
        r#"{{"kind":"wire","ty":{},"value":{},"encoded":{encoded},"bound":{}}}"#,
        T::ty(),
        value.val(),
        json(T::MAX_ENCODED_BYTES),
    )
}

struct Face;
type Cutter = WorthQueryName<Face>;

/// A bounded discriminator: a cutter, a step and an optional side.
struct Cut {
    cutter: Cutter,
    step: u64,
    side: Option<bool>,
}

impl ApplicationNameDiscriminator for Cut {
    const DISCRIMINATOR_IDENTITY: &'static str = "worth.trace.cut.v1";
    const MAX_BODY_BYTES: Option<usize> = bound_sum(&[
        field_bound("cutter", <Cutter as Value>::MAX_ENCODED_BYTES),
        field_bound("step", <u64 as Value>::MAX_ENCODED_BYTES),
        field_bound("side", <Option<bool> as Value>::MAX_ENCODED_BYTES),
    ]);
    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder
            .field("cutter", &self.cutter)
            .field("step", &self.step)
            .field("side", &self.side);
    }
}

/// A text-carrying discriminator: never bounded.
struct Labelled {
    label: String,
}

impl ApplicationNameDiscriminator for Labelled {
    const DISCRIMINATOR_IDENTITY: &'static str = "worth.trace.label.v1";
    const MAX_BODY_BYTES: Option<usize> =
        bound_sum(&[field_bound("label", <String as Value>::MAX_ENCODED_BYTES)]);
    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.field("label", &self.label);
    }
}

fn disc_line<D: ApplicationNameDiscriminator>(
    value: &D,
    fields: &[(&str, String, String)],
) -> String {
    let rows: Vec<String> = fields
        .iter()
        .map(|(name, ty, _)| format!("[{},{ty}]", name.len()))
        .collect();
    let values: Vec<&str> = fields.iter().map(|(_, _, value)| value.as_str()).collect();
    format!(
        r#"{{"kind":"disc","identity":{},"fields":[{}],"values":[{}],"encoded":{},"bound":{}}}"#,
        D::DISCRIMINATOR_IDENTITY.len(),
        rows.join(","),
        values.join(","),
        value.canonical_bytes().len(),
        json(max_encoded_bytes::<D>()),
    )
}

#[test]
fn discriminator_wire_lengths_and_bounds_trace_against_the_proved_model() {
    let cutter = || Cutter::from_admitted_digest([7; 32]);
    let mut trace = String::new();
    for line in [
        value_line(true),
        value_line(u64::MAX),
        value_line(cutter()),
        value_line(String::from("left side")),
        value_line(None::<u64>),
        value_line(Some(Some(false))),
        value_line((cutter(), Some(3_u64))),
        value_line((false, String::new())),
        value_line(Some((cutter(), (true, 9_u64)))),
    ] {
        let _ = writeln!(trace, "{line}");
    }
    for side in [None, Some(true)] {
        let cut = Cut {
            cutter: cutter(),
            step: 4,
            side,
        };
        let fields = [
            ("cutter", Cutter::ty(), cut.cutter.val()),
            ("step", u64::ty(), cut.step.val()),
            ("side", <Option<bool>>::ty(), cut.side.val()),
        ];
        let _ = writeln!(trace, "{}", disc_line(&cut, &fields));
    }
    let labelled = Labelled {
        label: "a label of any length".into(),
    };
    let fields = [("label", String::ty(), labelled.label.val())];
    let _ = writeln!(trace, "{}", disc_line(&labelled, &fields));
    if let Ok(dir) = std::env::var("WORTH_PERSISTENT_NAMING_TRACE_DIR") {
        std::fs::write(format!("{dir}/wire_bound.ndjson"), trace).unwrap();
    }
}
