//! Stored names and targets: one canonical text per value and kind, and a
//! typed refusal for everything that is not that text.
use worth_foundational::facade::{AspectValue, InternedString};

use super::stored_name::{decode_name, decode_target, encode_name, encode_target};
use super::{
    EntityNameSubject, NameSelection, NamedTarget, NamedTargetDelegate, RelationNameSubject,
    StoredNameDenial, Unowned, WorthQueryName, WorthQueryNamedKind,
};
use crate::application_schema::fragment::{field, Entity, NativeScalars, Record, Relation, Shape};
use crate::application_schema::{
    decode_canonical_value, encode_canonical_value, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeDenial,
    NamedTargetApplicationValueBinding, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds,
    WorthQueryNameApplicationValueBinding,
};

struct Face;
struct Edge;
struct Seam;
impl Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = (Seam,);
}
impl Entity for Edge {
    const NAME: &'static str = "Edge";
    type Aspects = ();
    type Relations = ();
}
impl Relation for Seam {
    const NAME: &'static str = "Seam";
    type To = Face;
}
impl WorthQueryNamedKind for Face {
    type Subject = EntityNameSubject;
    type OwnerScope = Unowned;
}
impl WorthQueryNamedKind for Edge {
    type Subject = EntityNameSubject;
    type OwnerScope = Unowned;
}
impl WorthQueryNamedKind for Seam {
    type Subject = RelationNameSubject;
    type OwnerScope = Unowned;
}

fn name<K>(byte: u8) -> WorthQueryName<K> {
    WorthQueryName::from_admitted_digest([byte; 32])
}

fn bounds() -> ValueDecodeBounds {
    ValueDecodeBounds::unrestricted()
}

#[test]
fn a_name_round_trips_through_exactly_one_text_per_kind() {
    let face = name::<Face>(0xab);
    let text = encode_name(&face).unwrap();
    assert_eq!(text, format!("wqn1.e.Face.{}", "ab".repeat(32)));
    assert_eq!(decode_name::<Face>(&text), Ok(face));
    let seam = encode_name(&name::<Seam>(0x01)).unwrap();
    assert_eq!(seam, format!("wqn1.r.Seam.{}", "01".repeat(32)));
    assert_eq!(decode_name::<Seam>(&seam), Ok(name::<Seam>(0x01)));
}

#[test]
fn text_stored_as_another_kind_or_version_refuses_instead_of_reinterpreting() {
    let digest = "ab".repeat(32);
    let face = format!("wqn1.e.Face.{digest}");
    assert_eq!(
        decode_name::<Edge>(&face),
        Err(StoredNameDenial::SubjectMismatch)
    );
    assert_eq!(
        decode_name::<Seam>(&face),
        Err(StoredNameDenial::SubjectKindMismatch)
    );
    assert_eq!(
        decode_name::<Face>(&format!("wqn2.e.Face.{digest}")),
        Err(StoredNameDenial::UnsupportedVersion { observed: 2 })
    );
    // A target is never read as a bare name, nor a name as a target.
    assert_eq!(
        decode_name::<Face>(&format!("wqt1.e.Face.{digest}.exact")),
        Err(StoredNameDenial::Malformed)
    );
    assert_eq!(
        decode_target::<Face>(&face),
        Err(StoredNameDenial::Malformed)
    );
}

#[test]
fn every_malformed_text_refuses_typed() {
    let digest = "ab".repeat(32);
    let malformed = [
        String::new(),
        "wqn1".to_owned(),
        format!("wqn01.e.Face.{digest}"),
        format!("wqn.e.Face.{digest}"),
        format!("wqnx.e.Face.{digest}"),
        format!("wqn1.g.Face.{digest}"),
        format!("wqn1.e..{digest}"),
        format!("wqn1.e.Face.{}", "AB".repeat(32)),
        format!("wqn1.e.Face.{}", "ab".repeat(31)),
        format!("wqn1.e.Face.{}", "ab".repeat(33)),
        format!("wqn1.e.Face.{}zz", "ab".repeat(31)),
        format!("wqn1.e.Face.{digest}.extra"),
        format!(" wqn1.e.Face.{digest}"),
    ];
    for text in malformed {
        assert_eq!(
            decode_name::<Face>(&text),
            Err(StoredNameDenial::Malformed),
            "{text:?}"
        );
    }
}

#[test]
fn a_target_round_trips_with_each_selection() {
    let face = name::<Face>(0x10);
    let carriers = NameSelection::bounded_by_carriers([
        name::<Edge>(0x30).erased(),
        name::<Edge>(0x20).erased(),
    ])
    .unwrap();
    let side = NameSelection::keep_side(&Side(7)).unwrap();
    let digest = "10".repeat(32);
    for (selection, tail) in [
        (NameSelection::Exact, "exact".to_owned()),
        (NameSelection::AllDescendants, "all".to_owned()),
        (
            carriers,
            format!("carriers.{}.{}", "20".repeat(32), "30".repeat(32)),
        ),
    ] {
        let target = NamedTarget::new(face, selection);
        let text = encode_target(&target).unwrap();
        assert_eq!(text, format!("wqt1.e.Face.{digest}.{tail}"));
        assert_eq!(decode_target::<Face>(&text), Ok(target));
    }
    let target = NamedTarget::new(face, side);
    let text = encode_target(&target).unwrap();
    assert!(text.starts_with(&format!("wqt1.e.Face.{digest}.side.")));
    assert_eq!(decode_target::<Face>(&text), Ok(target));
}

#[test]
fn a_target_selection_outside_canonical_form_refuses() {
    let head = format!("wqt1.e.Face.{}", "10".repeat(32));
    let (low, high) = ("20".repeat(32), "30".repeat(32));
    for (text, denial) in [
        (
            format!("{head}.carriers"),
            StoredNameDenial::NonCanonicalSelection,
        ),
        (
            format!("{head}.carriers.{high}.{low}"),
            StoredNameDenial::NonCanonicalSelection,
        ),
        (
            format!("{head}.carriers.{low}.{low}"),
            StoredNameDenial::NonCanonicalSelection,
        ),
        (
            format!("{head}.carriers.{low}.zz"),
            StoredNameDenial::Malformed,
        ),
        (format!("{head}.side"), StoredNameDenial::Malformed),
        (format!("{head}.side."), StoredNameDenial::Malformed),
        (format!("{head}.side.abc"), StoredNameDenial::Malformed),
        (format!("{head}.side.ab.cd"), StoredNameDenial::Malformed),
        (format!("{head}.exact.all"), StoredNameDenial::Malformed),
        (format!("{head}.nearest"), StoredNameDenial::Malformed),
        (head.clone(), StoredNameDenial::Malformed),
    ] {
        assert_eq!(decode_target::<Face>(&text), Err(denial), "{text:?}");
    }
    assert_eq!(
        decode_target::<Edge>(&format!("{head}.exact")),
        Err(StoredNameDenial::SubjectMismatch)
    );
}

struct Side(u8);
impl super::ApplicationNameDiscriminator for Side {
    const DISCRIMINATOR_IDENTITY: &'static str = "worth.tests.stored-name.side.v1";
    fn encode_canonical(&self, encoder: &mut super::ApplicationNameDiscriminatorEncoder) {
        encoder.field("side", &self.0);
    }
}

/// A generic owner value names its admitted delegate; an alias needs none.
type FaceSupport = WorthQueryName<Face>;
#[derive(Debug, Eq, PartialEq, crate::application_schema::WorthValue)]
#[worth_value(crate = "crate::application_schema")]
struct Chamfer {
    #[worth_value(delegate = NamedTargetDelegate<Edge>)]
    target: NamedTarget<Edge>,
    support: Option<FaceSupport>,
    radius: u32,
}

#[test]
fn a_derived_canonical_value_carries_names_and_refuses_another_kind() {
    let chamfer = Chamfer {
        target: NamedTarget::all_descendants(name(0x44)),
        support: Some(name(0x55)),
        radius: 3,
    };
    let wire = encode_canonical_value(&chamfer, bounds()).unwrap();
    assert_eq!(
        decode_canonical_value::<Chamfer>(&wire, bounds()),
        Ok(chamfer)
    );

    let face = encode_canonical_value(&name::<Face>(0x55), bounds()).unwrap();
    assert_eq!(face, format!("\"wqn1.e.Face.{}\"", "55".repeat(32)));
    assert_eq!(
        decode_canonical_value::<WorthQueryName<Edge>>(&face, bounds()),
        Err(ValueCodecDenial::Malformed)
    );
    assert_eq!(
        decode_canonical_value::<WorthQueryName<Face>>(&face.replace("wqn1", "wqn9"), bounds()),
        Err(ValueCodecDenial::UnsupportedVersion {
            expected: 1,
            observed: 9
        })
    );
    let unordered = format!(
        "\"wqt1.e.Edge.{}.carriers.{}.{}\"",
        "44".repeat(32),
        "30".repeat(32),
        "20".repeat(32)
    );
    assert_eq!(
        decode_canonical_value::<NamedTarget<Edge>>(&unordered, bounds()),
        Err(ValueCodecDenial::NonCanonical)
    );
    assert!(decode_canonical_value::<WorthQueryName<Face>>("7", bounds()).is_err());
}

type FaceName = WorthQueryNameApplicationValueBinding<Face>;
type EdgeName = WorthQueryNameApplicationValueBinding<Edge>;

fn reason(denial: ApplicationValueDecodeDenial) -> &'static str {
    match denial {
        ApplicationValueDecodeDenial::Validation(denial) => denial.reason(),
        other => panic!("expected a stored-name refusal, got {other:?}"),
    }
}

#[test]
fn a_name_column_decodes_only_its_own_kind_and_family() {
    let stored = FaceName::encode(&name(0x66)).unwrap();
    assert_eq!(
        stored,
        AspectValue::String(InternedString::Raw(format!(
            "wqn1.e.Face.{}",
            "66".repeat(32)
        )))
    );
    assert_eq!(FaceName::decode(&stored), Ok(name(0x66)));
    assert_eq!(
        reason(EdgeName::decode(&stored).unwrap_err()),
        StoredNameDenial::SubjectMismatch.reason()
    );
    let corrupted = AspectValue::String(InternedString::Raw("wqn1.e.Face.beef".to_owned()));
    assert_eq!(
        reason(FaceName::decode(&corrupted).unwrap_err()),
        StoredNameDenial::Malformed.reason()
    );
    assert!(matches!(
        FaceName::decode(&AspectValue::UInt64(7)),
        Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch { .. })
    ));
    let target = NamedTarget::exact(name::<Edge>(0x77));
    let stored = NamedTargetApplicationValueBinding::<Edge>::encode(&target).unwrap();
    assert_eq!(
        NamedTargetApplicationValueBinding::<Edge>::decode(&stored),
        Ok(target)
    );
    assert_eq!(
        reason(NamedTargetApplicationValueBinding::<Face>::decode(&stored).unwrap_err()),
        StoredNameDenial::SubjectMismatch.reason()
    );
}

struct ChamferRow {
    target: NamedTarget<Edge>,
    support: Option<WorthQueryName<Face>>,
}

crate::worth_query_native_scalar_bindings!(Codecs);
impl Record<ChamferRow> for Codecs {
    fn record() -> impl Shape<ChamferRow, Self> {
        field("Target", |row: &ChamferRow| row.target.clone())
            .and("Support", |row: &ChamferRow| row.support)
            .build(|target, support, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ChamferRow { target, support })
            })
    }
}

#[test]
fn a_record_stores_names_through_the_stock_binding_set() {
    fn registered<T: 'static, Set: Record<T>>() {}
    registered::<WorthQueryName<Seam>, NativeScalars>();
    let row = ChamferRow {
        target: NamedTarget::exact(name(0x88)),
        support: None,
    };
    let record = <Codecs as Record<ChamferRow>>::record();
    let mut stored = record.encode(&row, "").unwrap();
    assert_eq!(stored.len(), 1, "an absent optional name stores no column");
    let mut admit = |_: usize, _: usize| Ok::<(), ValueCodecDenial>(());
    let read = record.decode(&stored, "", &mut admit).unwrap();
    assert_eq!((read.target, read.support), (row.target.clone(), None));

    stored.insert(
        "Support".to_owned(),
        AspectValue::String(InternedString::Raw(format!(
            "wqn1.e.Edge.{}",
            "99".repeat(32)
        ))),
    );
    assert!(
        record.decode(&stored, "", &mut admit).is_err(),
        "an edge name stored in a face column refuses the whole row"
    );
}
