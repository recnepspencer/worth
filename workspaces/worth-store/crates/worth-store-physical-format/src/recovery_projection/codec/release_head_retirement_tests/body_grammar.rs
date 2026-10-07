//! The retirement body is exactly its grammar: the optional result root flag
//! is absent or present and nothing follows the node writes.

use super::*;

fn one_survivor() -> WireBody {
    let body = WireBody::of(&retirement(vec![terminal_head(), survivor()]));
    assert!(decode(&body.projection_bytes()).is_ok());
    body
}

#[test]
fn nothing_follows_the_node_writes_of_a_retirement_body() {
    let bytes = one_survivor().projection_bytes_with_body(|body| body.push(0));
    assert_eq!(decode(&bytes), Err(MALFORMED));
}

#[test]
fn the_result_root_flag_of_a_retirement_is_exactly_absent_or_present() {
    // tree identity, source basis field, source root, next block, mutation
    // tag, expected prior entry.
    let before_entry = 8 + 8 + basis().encode().len() + HeadRef::ENCODED_BYTES + 8 + 1;
    let flag = before_entry + HeadEntry::ENCODED_BYTES;
    let bytes = one_survivor().projection_bytes_with_body(|body| {
        assert_eq!(body[flag], 1, "the surviving head keeps a result root");
        body[flag] = 2;
    });
    assert_eq!(decode(&bytes), Err(MALFORMED));
}
