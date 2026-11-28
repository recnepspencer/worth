//! Canonical admitted request-binding digest covering installed graph contract,
//! query identity, canonical parameter bindings, resolved scope identity, and
//! authenticated external principal identity.
//!
//! Shared identically by the query executor (`read_execution.rs`, embedded in
//! every [`super::WorthQueryApplicationRootWindowProgress`] receipt) and Query
//! Host (`open_historical_query_cursor` / `resume_historical_query_cursor`), so
//! two requests sharing the same graph contract and snapshot with different
//! parameters, scopes, or principals can never cross-substitute page receipts.

use sha2::{Digest, Sha256};
use worth_foundational::facade::{AspectValue, InternedString};

const REQUEST_BINDING_DOMAIN: &[u8] = b"worth.query.window.request-binding.v2";

/// Computes the canonical 256-bit admitted request-binding digest for a query
/// window execution.
///
/// `parameter_bindings` are canonicalized (sorted by parameter name) so callers
/// supplying parameters in any order produce the exact same digest.
pub fn compute_admitted_request_binding_digest<'a>(
    contract_digest: &[u8; 32],
    query_identity: &[u8; 32],
    parameter_bindings: &[(&'a str, &'a AspectValue)],
    scope_entity: &str,
    scope_value: Option<&AspectValue>,
    principal_issuer: &str,
    principal_subject: &str,
) -> [u8; 32] {
    let mut sorted_params: Vec<(&str, &AspectValue)> = parameter_bindings.to_vec();
    sorted_params.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut hasher = Sha256::new();
    hasher.update(REQUEST_BINDING_DOMAIN);
    hasher.update(contract_digest);
    hasher.update(query_identity);
    hasher.update((sorted_params.len() as u64).to_le_bytes());
    for (name, value) in sorted_params {
        hash_str(&mut hasher, name);
        hash_aspect_value(&mut hasher, value);
    }
    hash_str(&mut hasher, scope_entity);
    match scope_value {
        Some(value) => {
            hasher.update([1u8]);
            hash_aspect_value(&mut hasher, value);
        }
        None => {
            hasher.update([0u8]);
        }
    }
    hash_str(&mut hasher, principal_issuer);
    hash_str(&mut hasher, principal_subject);
    hasher.finalize().into()
}

fn hash_str(h: &mut Sha256, s: &str) {
    h.update((s.len() as u64).to_le_bytes());
    h.update(s.as_bytes());
}

fn hash_tag(h: &mut Sha256, tag: u8, bytes: &[u8]) {
    h.update([tag]);
    h.update(bytes);
}

fn hash_interned_string(h: &mut Sha256, s: &InternedString) {
    match s {
        InternedString::Raw(text) => {
            h.update([0u8]);
            hash_str(h, text);
        }
        InternedString::Stored(text) => {
            h.update([0u8]);
            hash_str(h, text.as_str());
        }
        InternedString::Symbol(sym) => {
            h.update([1u8]);
            h.update(sym.0.to_le_bytes());
        }
    }
}

fn hash_aspect_value(h: &mut Sha256, value: &AspectValue) {
    match value {
        AspectValue::Null => h.update([0]),
        AspectValue::Bool(v) => h.update([1, u8::from(*v)]),
        AspectValue::Int8(v) => h.update([2, *v as u8]),
        AspectValue::Int16(v) => hash_tag(h, 3, &v.to_le_bytes()),
        AspectValue::Int32(v) => hash_tag(h, 4, &v.to_le_bytes()),
        AspectValue::Int64(v) => hash_tag(h, 5, &v.to_le_bytes()),
        AspectValue::UInt8(v) => h.update([6, *v]),
        AspectValue::UInt16(v) => hash_tag(h, 7, &v.to_le_bytes()),
        AspectValue::UInt32(v) => hash_tag(h, 8, &v.to_le_bytes()),
        AspectValue::UInt64(v) => hash_tag(h, 9, &v.to_le_bytes()),
        AspectValue::Float32(v) => hash_tag(h, 10, &v.bits().to_le_bytes()),
        AspectValue::Float64(v) => hash_tag(h, 11, &v.bits().to_le_bytes()),
        AspectValue::Decimal(v) => {
            h.update([12]);
            hash_str(h, v.as_str());
        }
        AspectValue::BigInt(v) => {
            h.update([13]);
            hash_str(h, v.as_str());
        }
        AspectValue::Rational(v) => {
            h.update([14]);
            hash_str(h, v.numerator.as_str());
            hash_str(h, v.denominator.as_str());
        }
        AspectValue::String(s) => {
            h.update([15]);
            hash_interned_string(h, s);
        }
        AspectValue::Uuid(u) => hash_tag(h, 16, u),
        AspectValue::Date(d) => hash_tag(h, 17, &d.days_from_unix_epoch.to_le_bytes()),
        AspectValue::Time(t) => hash_tag(h, 18, &t.nanos_since_midnight.to_le_bytes()),
        AspectValue::Timestamp(ts) => hash_tag(h, 19, &ts.micros_since_unix_epoch.to_le_bytes()),
        AspectValue::TimestampTz(tz) => {
            hash_tag(h, 20, &tz.utc_micros_since_unix_epoch.to_le_bytes());
            h.update(tz.offset_minutes.to_le_bytes());
        }
        AspectValue::EntityRef(id) => {
            hash_tag(h, 21, &id.partition_id.0.to_le_bytes());
            h.update(id.local_slot.0.to_le_bytes());
            h.update(id.generation.0.to_le_bytes());
        }
        AspectValue::ContentRef(id) => hash_tag(h, 22, &id.0),
        AspectValue::InlineBytes(b) => {
            hash_tag(h, 23, &(b.as_bytes().len() as u64).to_le_bytes());
            h.update(b.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compute_admitted_request_binding_digest;
    use worth_foundational::facade::{AspectValue, InternedString, Symbol};

    #[test]
    fn empty_text_and_zero_symbol_produce_distinct_request_binding_digests() {
        let contract = [0x11u8; 32];
        let query = [0x22u8; 32];
        let empty_text = AspectValue::String(InternedString::Raw(String::new()));
        let zero_symbol = AspectValue::String(InternedString::Symbol(Symbol(0)));
        let one_symbol = AspectValue::String(InternedString::Symbol(Symbol(1)));
        let one_byte_text = AspectValue::String(InternedString::Raw("\x01".to_owned()));

        let d_empty = compute_admitted_request_binding_digest(
            &contract,
            &query,
            &[("filter", &empty_text)],
            "Project",
            None,
            "iss",
            "sub",
        );
        let d_sym0 = compute_admitted_request_binding_digest(
            &contract,
            &query,
            &[("filter", &zero_symbol)],
            "Project",
            None,
            "iss",
            "sub",
        );
        let d_sym1 = compute_admitted_request_binding_digest(
            &contract,
            &query,
            &[("filter", &one_symbol)],
            "Project",
            None,
            "iss",
            "sub",
        );
        let d_text1 = compute_admitted_request_binding_digest(
            &contract,
            &query,
            &[("filter", &one_byte_text)],
            "Project",
            None,
            "iss",
            "sub",
        );
        assert_ne!(d_empty, d_sym0);
        assert_ne!(d_sym0, d_sym1);
        assert_ne!(d_sym1, d_text1);

        let d_scope_empty = compute_admitted_request_binding_digest(
            &contract,
            &query,
            &[],
            "Project",
            Some(&empty_text),
            "iss",
            "sub",
        );
        let d_scope_sym0 = compute_admitted_request_binding_digest(
            &contract,
            &query,
            &[],
            "Project",
            Some(&zero_symbol),
            "iss",
            "sub",
        );
        assert_ne!(d_scope_empty, d_scope_sym0);
    }
}
