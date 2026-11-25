//! Shared kinds, roles and discriminators for the recorded-birth decode
//! tests. Discriminators are encoded by `worth_query_name_discriminator!`
//! and decoded by hand, exactly as an application without the derive does.

use super::{
    AdmittedNameReferences, ApplicationNameDiscriminatorDecode,
    ApplicationNameDiscriminatorDecodeDenial as Denial, ApplicationNameDiscriminatorDecoder,
    ApplicationNameSubjectKind, EntityNameSubject, NameRef, NestedDiscriminator, RecordedNameBirth,
    Unowned, WorthQueryName, WorthQueryNameRole, WorthQueryNamedKind,
};
use crate::application_schema::fragment::Entity;

type Decoded<T> = Result<T, Denial>;

pub(super) struct Face;
impl Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = ();
}
impl WorthQueryNamedKind for Face {
    type Subject = EntityNameSubject;
    type OwnerScope = Unowned;
}

pub(super) struct Edge;
impl Entity for Edge {
    const NAME: &'static str = "Edge";
    type Aspects = ();
    type Relations = ();
}
impl WorthQueryNamedKind for Edge {
    type Subject = EntityNameSubject;
    type OwnerScope = Unowned;
}

pub(super) fn name(byte: u8) -> WorthQueryName<Face> {
    WorthQueryName::from_admitted_digest([byte; 32])
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Role {
    Authored,
    Instance(u32),
}

impl WorthQueryNameRole for Role {
    const ROLE_ORDER: &'static [&'static str] = &["Authored", "Instance"];
    const ROLE_DEPENDENCIES: &'static [(&'static str, &'static str)] = &[];
    fn role_name(&self) -> &'static str {
        match self {
            Self::Authored => "Authored",
            Self::Instance(_) => "Instance",
        }
    }
    fn role_index(&self) -> Option<u32> {
        match self {
            Self::Authored => None,
            Self::Instance(index) => Some(*index),
        }
    }
    fn from_role_parts(role: &str, index: Option<u32>) -> Option<Self> {
        match (role, index) {
            ("Authored", None) => Some(Self::Authored),
            ("Instance", Some(index)) => Some(Self::Instance(index)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Side {
    pub(super) side: u8,
}
crate::worth_query_name_discriminator! {
    struct Side { identity: "test.side.v1", fields: [side] }
}
impl ApplicationNameDiscriminatorDecode for Side {
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self> {
        Ok(Self {
            side: decoder.field("side")?,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Every {
    pub(super) flag: bool,
    pub(super) small: i8,
    pub(super) medium: i32,
    pub(super) large: u64,
    pub(super) signed: i64,
    pub(super) word: u16,
    pub(super) text: String,
    pub(super) uuid: [u8; 16],
    pub(super) maybe: Option<u32>,
    pub(super) pair: (u8, String),
    pub(super) cutter: NameRef<Face>,
    pub(super) edge: WorthQueryName<Face>,
    pub(super) nested: Side,
    pub(super) nested_option: Option<NestedDiscriminator<Side>>,
}
crate::worth_query_name_discriminator! {
    struct Every {
        identity: "test.every.v1",
        fields: [flag, small, medium, large, signed, word, text, uuid, maybe, pair, cutter, edge,
            nested, nested_option]
    }
}
impl ApplicationNameDiscriminatorDecode for Every {
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self> {
        Ok(Self {
            flag: decoder.field("flag")?,
            small: decoder.field("small")?,
            medium: decoder.field("medium")?,
            large: decoder.field("large")?,
            signed: decoder.field("signed")?,
            word: decoder.field("word")?,
            text: decoder.field("text")?,
            uuid: decoder.field("uuid")?,
            maybe: decoder.field("maybe")?,
            pair: decoder.field("pair")?,
            cutter: decoder.field("cutter")?,
            edge: decoder.field("edge")?,
            nested: decoder.field("nested")?,
            nested_option: decoder.field("nested_option")?,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Cut {
    Sided { cutter: NameRef<Face>, side: u8 },
    Seam { index: u32 },
}
crate::worth_query_name_discriminator! {
    enum Cut { identity: "test.cut.v1", variants: { Sided { cutter, side }, Seam { index } } }
}
impl ApplicationNameDiscriminatorDecode for Cut {
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self> {
        match decoder.variant()? {
            "Sided" => Ok(Self::Sided {
                cutter: decoder.field("cutter")?,
                side: decoder.field("side")?,
            }),
            "Seam" => Ok(Self::Seam {
                index: decoder.field("index")?,
            }),
            other => Err(decoder.unknown_variant(other)),
        }
    }
}

/// The same identity and wire form as [`Cut::Sided`], declared over edges.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct EdgeCut {
    pub(super) cutter: NameRef<Edge>,
    pub(super) side: u8,
}
impl super::ApplicationNameDiscriminator for EdgeCut {
    const DISCRIMINATOR_IDENTITY: &'static str = "test.cut.v1";
    fn encode_canonical(&self, encoder: &mut super::ApplicationNameDiscriminatorEncoder) {
        encoder
            .variant("Sided")
            .field("cutter", &self.cutter)
            .field("side", &self.side);
    }
}
impl ApplicationNameDiscriminatorDecode for EdgeCut {
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self> {
        match decoder.variant()? {
            "Sided" => Ok(Self {
                cutter: decoder.field("cutter")?,
                side: decoder.field("side")?,
            }),
            other => Err(decoder.unknown_variant(other)),
        }
    }
}

/// Other types under the same identities, to forge mismatched bodies.
pub(super) enum CutImpostor {
    Unknown,
}
crate::worth_query_name_discriminator! {
    enum CutImpostor { identity: "test.cut.v1", variants: { Unknown {} } }
}
pub(super) struct SideImpostor {
    pub(super) other: u8,
}
crate::worth_query_name_discriminator! {
    struct SideImpostor { identity: "test.side.v1", fields: [other] }
}

pub(super) fn every() -> Every {
    Every {
        flag: true,
        small: -3,
        medium: i32::MIN,
        large: u64::MAX,
        signed: -1,
        word: 7,
        text: "piece".into(),
        uuid: [9; 16],
        maybe: Some(4),
        pair: (1, "a".into()),
        cutter: name(1).into(),
        edge: name(2),
        nested: Side { side: 1 },
        nested_option: Some(NestedDiscriminator(Side { side: 0 })),
    }
}

/// A recorded `Face` birth whose read admitted each `admitted` byte's
/// digest as a `Face` name.
pub(super) fn birth(role: &str, bytes: Vec<u8>, admitted: &[u8]) -> RecordedNameBirth<Face> {
    let faces = admitted.iter().map(|byte| {
        (
            [*byte; 32],
            (ApplicationNameSubjectKind::Entity, "Face".to_owned()),
        )
    });
    birth_admitting(role, bytes, faces.collect())
}

pub(super) fn birth_admitting(
    role: &str,
    bytes: Vec<u8>,
    admitted: AdmittedNameReferences,
) -> RecordedNameBirth<Face> {
    RecordedNameBirth::from_admitted_parts(
        name(0xee),
        "feature/instance[3]".into(),
        role.into(),
        vec![name(0xaa)],
        bytes,
        admitted,
    )
}
