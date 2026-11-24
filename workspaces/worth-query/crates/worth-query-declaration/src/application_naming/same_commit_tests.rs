use std::sync::Arc;

use super::same_commit::{SameCommitNameReference, SAME_COMMIT_NAME_PLACEHOLDER};
use super::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorValue, NestedDiscriminator, WorthQueryName,
    WorthQuerySameCommitName,
};

struct Face;

/// One discriminator shape over either kind of name reference.
struct CutBy<R> {
    cutter: R,
    side: u8,
}

impl<R: ApplicationNameDiscriminatorValue + 'static> ApplicationNameDiscriminator for CutBy<R> {
    const DISCRIMINATOR_IDENTITY: &'static str = "worth.tests.cut-by.v1";

    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder
            .field("cutter", &self.cutter)
            .field("side", &self.side);
    }
}

/// A reference directly, inside an option and a pair, and one level nested.
fn encode<R: ApplicationNameDiscriminatorValue + Clone + 'static>(
    first: R,
    second: R,
) -> ApplicationNameDiscriminatorEncoder {
    let mut encoder = ApplicationNameDiscriminatorEncoder::new("worth.tests.outer.v1");
    encoder
        .field("direct", &first)
        .field("optional", &Some(second.clone()))
        .field(
            "nested",
            &NestedDiscriminator(CutBy {
                cutter: first.clone(),
                side: 4,
            }),
        )
        .field("pair", &(second, 9_u32));
    encoder
}

fn fresh(candidate: &Arc<()>, output: usize) -> WorthQuerySameCommitName<Face> {
    WorthQuerySameCommitName::issued(SameCommitNameReference::issued(
        Arc::clone(candidate),
        output,
    ))
}

#[test]
fn a_same_commit_name_takes_the_wire_form_of_the_committed_name_it_stands_for() {
    let candidate = Arc::new(());
    let (bytes, committed_refs, slots) =
        encode(fresh(&candidate, 0), fresh(&candidate, 1)).finish_with_same_commit_names();
    assert!(committed_refs.is_empty());
    let designated: Vec<_> = slots.iter().map(|slot| slot.reference().output()).collect();
    assert_eq!(
        designated,
        vec![0, 1, 0, 1],
        "one slot per reference, in order"
    );
    assert!(slots
        .iter()
        .all(|slot| slot.reference().issued_by(&candidate)));
    assert!(!slots[0].reference().issued_by(&Arc::new(())));

    // Every slot marks exactly an unwritten placeholder, at any nesting.
    let digests = [[0x11_u8; 32], [0x22_u8; 32]];
    let mut written = bytes;
    for slot in &slots {
        let end = slot.offset() + 32;
        assert_eq!(written[slot.offset()..end], SAME_COMMIT_NAME_PLACEHOLDER);
        written[slot.offset()..end].copy_from_slice(&digests[slot.reference().output()]);
    }

    // Written, the stream is the one the committed names encode to.
    let name = |digest| WorthQueryName::<Face>::from_admitted_digest(digest);
    let (committed, refs) = encode(name(digests[0]), name(digests[1])).finish_with_references();
    assert_eq!(written, committed);
    assert_eq!(refs, vec![digests[0], digests[1], digests[0], digests[1]]);
}
