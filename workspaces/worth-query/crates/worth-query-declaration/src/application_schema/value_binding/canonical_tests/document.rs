//! Compile the document macro arm against an owner that seals body and original text.
use super::*;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct SpanDocument {
    value: Span,
    text: String,
}

impl CanonicalDocumentCodec for Span {
    type Document = SpanDocument;
    // This fixture deliberately exercises an explicit domain wire policy.
    const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(64, 64, 4, 4);
    fn encoded(document: &SpanDocument) -> &str {
        &document.text
    }
    fn read(raw: &str) -> Result<SpanDocument, ValueCodecDenial> {
        finish(decode_canonical_value(raw, Self::BOUNDS)?, raw)
    }
    fn read_admitted(
        raw: &str,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<SpanDocument, ValueCodecDenial> {
        finish(
            decode_canonical_value_with_admission(raw, Self::BOUNDS, admission)?,
            raw,
        )
    }
}

fn finish(value: Span, raw: &str) -> Result<SpanDocument, ValueCodecDenial> {
    positive(&value).map_err(|_| ValueCodecDenial::Malformed)?;
    let mut text = String::new();
    text.try_reserve_exact(raw.len())
        .map_err(|_| ValueCodecDenial::Allocation)?;
    text.push_str(raw);
    Ok(SpanDocument { value, text })
}

crate::worth_query_value_binding!(DocumentBinding for SpanDocument {
    identity: "tests.span.document.v1", native_envelope: Span,
});

#[test]
fn native_envelope_binding_preserves_sealed_text_policy_and_original_stop() {
    let raw = r#"{"length":17,"label":"δ"}"#;
    let document = Span::read(raw).unwrap();
    assert_eq!(DocumentBinding::validate(&document), Ok(()));
    let encoded = DocumentBinding::encode(&document).unwrap();
    assert_eq!(encoded, AspectValue::String(raw.into()));
    assert_eq!(DocumentBinding::decode(&encoded).unwrap(), document);
    assert_eq!(
        DocumentBinding::decode_admitted(&encoded, &mut |_, _| Ok(())).unwrap(),
        document
    );
    for cause in [ValueCodecDenial::Admission, ValueCodecDenial::Allocation] {
        let mut calls = 0;
        assert_eq!(
            DocumentBinding::decode_admitted(&encoded, &mut |_, _| {
                calls += 1;
                Err(cause)
            }),
            Err(ApplicationValueDecodeDenial::Canonical {
                binding_identity: DocumentBinding::IDENTITY,
                cause,
            })
        );
        assert_eq!(calls, 1);
    }

    // The fixture owner can construct a policy-invalid pair to probe the binding.
    let value = Span {
        length: 17,
        label: "x".repeat(64),
    };
    let text = encode_canonical_value(&value, ValueDecodeBounds::unrestricted()).unwrap();
    let oversized = SpanDocument { value, text };
    let refusal = ApplicationValueValidationDenial::rejected(
        DocumentBinding::IDENTITY,
        "native record exceeds its wire bound",
    );
    assert_eq!(DocumentBinding::validate(&oversized), Err(refusal.clone()));
    assert_eq!(
        DocumentBinding::encode(&oversized),
        Err(ApplicationValueEncodeDenial::Validation(refusal))
    );
}
