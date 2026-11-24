use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::lineage::data::naming::PersistentNameBirthRequest;
use crate::contracts::lineage::data::naming_denial::{
    DiscriminatorBasisDenialReason, LineageNameDenial,
};
use crate::lineage::authority::discriminator_scan::extract_embedded_name_refs;
use crate::lineage::authority::PersistentNameMintingAuthority;

use super::{entity_holder, face_registry};

const DOMAIN_TAG: &[u8] = b"worth.query.name-discriminator.v1\0";

fn push_len_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn begin_v1(identity: &str) -> Vec<u8> {
    let mut out = DOMAIN_TAG.to_vec();
    push_len_bytes(&mut out, identity.as_bytes());
    out
}

fn mint_expect_malformed(disc: Vec<u8>) {
    let registry = face_registry();
    let denial = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.malformed".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: disc,
            holder: entity_holder(1, 1),
        }],
    )
    .expect_err("malformed discriminator must refuse");
    assert_eq!(
        denial,
        LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "cut_piece".to_owned(),
            reason: DiscriminatorBasisDenialReason::MalformedDiscriminator,
        }
    );
}

#[test]
fn scanner_accepts_every_canonical_tag_nested_in_pair_and_deep_option_70() {
    assert_eq!(extract_embedded_name_refs(&[]), Ok(Vec::new()));

    let ref_a = [0x11; 32];
    let ref_b = [0x22; 32];
    let ref_c = [0x33; 32];

    // Build inner nested discriminator carrying VARIANT + FIELD(NAME_REF) + FIELD(u32).
    let mut inner_body = Vec::new();
    inner_body.push(0x01); // TAG_VARIANT
    push_len_bytes(&mut inner_body, b"InnerCut");
    inner_body.push(0x02); // TAG_FIELD
    push_len_bytes(&mut inner_body, b"inner_ref");
    inner_body.push(0x20); // TAG_NAME_REF
    inner_body.extend_from_slice(&ref_b);
    inner_body.push(0x02); // TAG_FIELD
    push_len_bytes(&mut inner_body, b"index");
    inner_body.push(0x17); // TAG_U32
    inner_body.extend_from_slice(&7_u32.to_be_bytes());

    let mut stream = begin_v1("worth.test.all_tags.v1");
    // TAG_VARIANT
    stream.push(0x01);
    push_len_bytes(&mut stream, b"AllTags");
    // TAG_FIELD -> TAG_BOOL(0), TAG_BOOL(1)
    for (name, flag) in [("b0", 0u8), ("b1", 1u8)] {
        stream.push(0x02);
        push_len_bytes(&mut stream, name.as_bytes());
        stream.push(0x10);
        stream.push(flag);
    }
    // All signed & unsigned integer tags (positional and named)
    stream.extend_from_slice(&[0x11, 0xfe]); // i8
    stream.extend_from_slice(&[0x12, 0x00, 0x02]); // i16
    stream.extend_from_slice(&[0x13, 0, 0, 0, 3]); // i32
    stream.extend_from_slice(&[0x14, 0, 0, 0, 0, 0, 0, 0, 4]); // i64
    stream.extend_from_slice(&[0x15, 5]); // u8
    stream.extend_from_slice(&[0x16, 0, 6]); // u16
    stream.extend_from_slice(&[0x17, 0, 0, 0, 7]); // u32
    stream.extend_from_slice(&[0x18, 0, 0, 0, 0, 0, 0, 0, 8]); // u64
                                                               // TAG_TEXT
    stream.push(0x19);
    push_len_bytes(&mut stream, b"hello-utf8");
    // TAG_UUID
    stream.push(0x1a);
    stream.extend_from_slice(&[0xab; 16]);
    // TAG_OPTION_NONE and TAG_OPTION_SOME(TAG_NAME_REF(ref_a))
    stream.push(0x30);
    stream.push(0x31);
    stream.push(0x20);
    stream.extend_from_slice(&ref_a);
    // TAG_FIELD "p" -> TAG_PAIR(TAG_NESTED_DISCRIMINATOR(inner_body), TAG_U32(99))
    stream.push(0x02);
    push_len_bytes(&mut stream, b"p");
    stream.push(0x32); // TAG_PAIR
    stream.push(0x33); // TAG_NESTED_DISCRIMINATOR
    push_len_bytes(&mut stream, b"worth.test.inner.v1");
    push_len_bytes(&mut stream, &inner_body);
    stream.push(0x17); // TAG_U32
    stream.extend_from_slice(&99_u32.to_be_bytes());
    // Trailing TAG_PAIR(TAG_NAME_REF(ref_c), TAG_OPTION_NONE)
    stream.push(0x32);
    stream.push(0x20);
    stream.extend_from_slice(&ref_c);
    stream.push(0x30);

    assert_eq!(
        extract_embedded_name_refs(&stream),
        Ok(vec![ref_a, ref_b, ref_c])
    );

    // Deep nesting: Option^70<u32> must parse without artificial depth limit.
    let mut deep = begin_v1("worth.test.deep.v1");
    deep.push(0x02);
    push_len_bytes(&mut deep, b"deep_opt");
    deep.extend(std::iter::repeat_n(0x31_u8, 70));
    deep.push(0x17);
    deep.extend_from_slice(&123_u32.to_be_bytes());
    assert_eq!(extract_embedded_name_refs(&deep), Ok(Vec::new()));
}

#[test]
fn scanner_refuses_non_v1_prefix_truncation_trailing_garbage_bad_bool_and_invalid_utf8() {
    // 1. Non-v1 prefix (`...v2\0`) carrying a TAG_NAME_REF must fail closed.
    let mut v2_smuggle = b"worth.query.name-discriminator.v2\0".to_vec();
    push_len_bytes(&mut v2_smuggle, b"worth.test.v2");
    v2_smuggle.push(0x20);
    v2_smuggle.extend_from_slice(&[0xee; 32]);
    assert_eq!(extract_embedded_name_refs(&v2_smuggle), Err(()));
    mint_expect_malformed(v2_smuggle);

    // Raw opaque label bytes without domain header also refuse as MalformedDiscriminator.
    assert_eq!(extract_embedded_name_refs(b"left"), Err(()));
    mint_expect_malformed(b"left".to_vec());

    // 2. Truncation at structural boundaries.
    let mut valid_nested = begin_v1("worth.test.trunc.v1");
    valid_nested.push(0x02);
    push_len_bytes(&mut valid_nested, b"n");
    valid_nested.push(0x33);
    push_len_bytes(&mut valid_nested, b"worth.test.inner.v1");
    let mut inner = vec![0x02];
    push_len_bytes(&mut inner, b"x");
    inner.push(0x17);
    inner.extend_from_slice(&1_u32.to_be_bytes());
    push_len_bytes(&mut valid_nested, &inner);

    for cut in [
        1,
        DOMAIN_TAG.len(),
        DOMAIN_TAG.len() + 4,
        valid_nested.len() - 1,
    ] {
        assert_eq!(extract_embedded_name_refs(&valid_nested[..cut]), Err(()));
    }
    mint_expect_malformed(valid_nested[..valid_nested.len() - 1].to_vec());

    // Truncated field inside nested body (length matches, but field lacks value).
    let mut bad_nested = begin_v1("worth.test.bad_nested.v1");
    bad_nested.push(0x33);
    push_len_bytes(&mut bad_nested, b"worth.test.inner.v1");
    let mut incomplete_inner = vec![0x02];
    push_len_bytes(&mut incomplete_inner, b"x");
    push_len_bytes(&mut bad_nested, &incomplete_inner);
    assert_eq!(extract_embedded_name_refs(&bad_nested), Err(()));

    // 3. Trailing garbage & unknown tag & item tag in value position.
    let mut trailing = valid_nested.clone();
    trailing.push(0xff);
    assert_eq!(extract_embedded_name_refs(&trailing), Err(()));
    mint_expect_malformed(trailing);

    let mut field_in_value = begin_v1("worth.test.bad_val.v1");
    field_in_value.push(0x02);
    push_len_bytes(&mut field_in_value, b"k");
    field_in_value.push(0x02); // TAG_FIELD invalid in value position
    push_len_bytes(&mut field_in_value, b"nested");
    assert_eq!(extract_embedded_name_refs(&field_in_value), Err(()));

    // 4. Invalid TAG_BOOL byte outside {0, 1}.
    for bad_bool in [2_u8, 255_u8] {
        let mut s = begin_v1("worth.test.bad_bool.v1");
        s.extend_from_slice(&[0x10, bad_bool]);
        assert_eq!(extract_embedded_name_refs(&s), Err(()));
        mint_expect_malformed(s);
    }

    // 5. Invalid UTF-8 in identity, TAG_VARIANT, TAG_FIELD, and TAG_TEXT.
    let mut bad_id = DOMAIN_TAG.to_vec();
    push_len_bytes(&mut bad_id, &[0xff, 0xfe]);
    assert_eq!(extract_embedded_name_refs(&bad_id), Err(()));

    for tag in [0x01_u8, 0x19_u8] {
        let mut s = begin_v1("worth.test.bad_utf8.v1");
        s.push(tag);
        push_len_bytes(&mut s, &[0xff, 0xfe]);
        assert_eq!(extract_embedded_name_refs(&s), Err(()));
    }
    let mut bad_field_utf8 = begin_v1("worth.test.bad_utf8.v1");
    bad_field_utf8.push(0x02);
    push_len_bytes(&mut bad_field_utf8, &[0xff, 0xfe]);
    bad_field_utf8.push(0x30);
    assert_eq!(extract_embedded_name_refs(&bad_field_utf8), Err(()));
    mint_expect_malformed(bad_field_utf8);
}
