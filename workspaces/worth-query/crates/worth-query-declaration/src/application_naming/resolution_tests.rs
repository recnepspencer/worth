use crate::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder, ErasedWorthQueryName,
    NameResolutionDenial, NameSelection, NameSelectionDenial, ResolvedName, WorthQueryName,
    NAME_DISCRIMINATOR_DOMAIN_TAG,
};

struct Face;

fn face(byte: u8) -> WorthQueryName<Face> {
    WorthQueryName::from_admitted_digest([byte; 32])
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CutSide {
    Left { cutter: WorthQueryName<Face> },
    Right { cutter: WorthQueryName<Face> },
}

crate::worth_query_name_discriminator! {
    enum CutSide {
        identity: "worth.tests.resolution.cut_side.v1",
        variants: {
            Left { cutter },
            Right { cutter },
        }
    }
}

struct MalformedSide;

impl ApplicationNameDiscriminator for MalformedSide {
    const DISCRIMINATOR_IDENTITY: &'static str = "bad identity";

    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.field("side", &1_u8);
    }
}

#[test]
fn erased_name_keeps_the_exact_digest_and_canonical_order() {
    let low = face(0x01);
    let high = face(0x02);
    assert_eq!(low.erased().as_bytes(), low.as_bytes());
    assert!(low.erased() < high.erased());
    assert_eq!(low.erased(), face(0x01).erased());
    assert_eq!(std::mem::size_of::<ErasedWorthQueryName>(), 32);
    let rendered = format!("{:?}", high.erased());
    assert!(rendered.contains(&"02".repeat(32)));
}

#[test]
fn bounded_by_carriers_sorts_and_deduplicates_independently_of_input_order() {
    let (a, b, c) = (face(0x30), face(0x10), face(0x20));
    let forward = NameSelection::bounded_by_carriers([a.erased(), b.erased(), c.erased()]).unwrap();
    let shuffled =
        NameSelection::bounded_by_carriers([c.erased(), a.erased(), b.erased(), a.erased()])
            .unwrap();
    assert_eq!(forward, shuffled);
    let NameSelection::BoundedByCarriers { carriers } = forward else {
        panic!("expected a carrier selection");
    };
    assert_eq!(
        carriers.as_slice(),
        &[b.erased(), c.erased(), a.erased()],
        "carriers are kept in ascending digest order without duplicates"
    );
}

#[test]
fn bounded_by_carriers_refuses_an_empty_carrier_list_instead_of_matching_everything() {
    assert_eq!(
        NameSelection::bounded_by_carriers([]),
        Err(NameSelectionDenial::EmptyCarrierSet)
    );
    assert_eq!(
        NameSelection::bounded_by_carriers(Vec::<ErasedWorthQueryName>::new()),
        Err(NameSelectionDenial::EmptyCarrierSet)
    );
    assert!(NameSelectionDenial::EmptyCarrierSet
        .to_string()
        .contains("no carrier"));
    let single = NameSelection::bounded_by_carriers([face(0x01).erased()]).unwrap();
    let NameSelection::BoundedByCarriers { carriers } = single else {
        panic!("expected a carrier selection");
    };
    assert_eq!(carriers.as_slice().len(), 1);
}

#[test]
fn keep_side_uses_the_canonical_encoder_and_separates_sides() {
    let cutter = face(0x44);
    let left = NameSelection::keep_side(&CutSide::Left { cutter }).unwrap();
    let right = NameSelection::keep_side(&CutSide::Right { cutter }).unwrap();
    assert_ne!(left, right);
    let NameSelection::KeepSide { discriminator } = &left else {
        panic!("expected a side selection");
    };
    let expected = CutSide::Left { cutter }.canonical_bytes();
    assert_eq!(discriminator.as_bytes(), expected.as_slice());
    assert!(discriminator
        .as_bytes()
        .starts_with(NAME_DISCRIMINATOR_DOMAIN_TAG));
}

#[test]
fn keep_side_refuses_a_malformed_discriminator_identity() {
    assert_eq!(
        NameSelection::keep_side(&MalformedSide),
        Err(NameSelectionDenial::InvalidDiscriminatorIdentity {
            discriminator_identity: "bad identity".to_owned(),
        })
    );
    let rendered = NameSelectionDenial::InvalidDiscriminatorIdentity {
        discriminator_identity: "bad identity".to_owned(),
    }
    .to_string();
    assert!(rendered.contains("bad identity"));
}

#[test]
fn resolution_vocabulary_compares_and_renders_without_kind_bounds() {
    let ambiguous = NameResolutionDenial::Ambiguous {
        candidates: vec![face(0x01), face(0x02)],
    };
    assert_eq!(ambiguous.clone(), ambiguous);
    assert_ne!(
        ambiguous,
        NameResolutionDenial::NoMatchingCandidate {
            live_candidates: vec![face(0x01), face(0x02)],
        }
    );
    assert_ne!(
        NameResolutionDenial::<Face>::BasisPruned,
        NameResolutionDenial::<Face>::UnknownName
    );
    assert!(ambiguous.to_string().contains("2 live candidates"));
    assert!(format!("{ambiguous:?}").starts_with("Ambiguous"));
    assert!(NameResolutionDenial::<Face>::BasisPruned
        .to_string()
        .contains("retention floor"));

    let plural: ResolvedName<u8> = ResolvedName::Plural(vec![1, 2]);
    assert_ne!(plural, ResolvedName::Exact(1));
    assert_eq!(
        ResolvedName::<u8>::Retired { at_commit: 4 },
        ResolvedName::Retired { at_commit: 4 }
    );
}
