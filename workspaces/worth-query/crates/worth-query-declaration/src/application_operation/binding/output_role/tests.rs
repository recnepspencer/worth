use crate::application_naming::{ApplicationNameDiscriminator, WorthQueryName};
use crate::application_operation::{
    ApplicationMutationLineageOutputContract, ApplicationMutationLineageOutputRoleDescriptor,
    ApplicationMutationLineageOutputRoleFamilyDescriptor, ApplicationMutationLineagePostureKind,
    ApplicationMutationLineagePostureSet, ApplicationMutationLineageSubjectKind,
    ApplicationMutationOutputPosture, ApplicationMutationOutputPostureSet,
    ErasedLineageOutputPosture, ErasedMergeKeeper, WorthQueryApplicationDeclaredLineageOutputRole,
    WorthQueryApplicationDeclaredLineageOutputRoleFamily, WorthQueryApplicationLineageOutputRole,
    WorthQueryApplicationLineageOutputRoleFamily, WorthQueryAtMostOneOutput,
    WorthQueryCreateOutput, WorthQueryEntityOutputSubject, WorthQueryExactlyOneOutput,
    WorthQueryLineageOutputPosture, WorthQueryLineagePostureDenial, WorthQueryMergeKeeper,
    WorthQueryMergeOutput, WorthQueryPreserveOutput, WorthQueryRelationOutputSubject,
    WorthQueryRetireOutput, WorthQuerySplitOutput,
};
use crate::application_schema::{
    fragment, ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
};

struct CadSchema;
impl ApplicationSchema for CadSchema {
    const OWNER: &'static str = "test";
    const NAME: &'static str = "cad";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!("declaration body not invoked in descriptor tests")
    }
}

struct Face;
impl fragment::Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = ();
}

struct Seam;
impl fragment::Relation for Seam {
    const NAME: &'static str = "Seam";
    type To = Face;
}

struct BooleanSplitMergeOutputs;
impl ApplicationMutationLineageOutputContract<CadSchema> for BooleanSplitMergeOutputs {
    const LINEAGE_ROLES: &'static [ApplicationMutationLineageOutputRoleDescriptor] = &[
        SplitFacePieceRole::DESCRIPTOR,
        MergedFaceRole::DESCRIPTOR,
        SplitSeamRole::DESCRIPTOR,
        MergedSeamRole::DESCRIPTOR,
        CreatedFaceRole::DESCRIPTOR,
        PreservedFaceRole::DESCRIPTOR,
        RetiredSeamRole::DESCRIPTOR,
    ];
    const LINEAGE_ROLE_FAMILIES: &'static [ApplicationMutationLineageOutputRoleFamilyDescriptor] =
        &[SplitFaceFamily::DESCRIPTOR, SplitSeamFamily::DESCRIPTOR];
}

struct SplitFacePieceRole;
impl WorthQueryApplicationLineageOutputRole for SplitFacePieceRole {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryEntityOutputSubject<Face>;
    type Action = WorthQuerySplitOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "split-face-piece";
}

struct MergedFaceRole;
impl WorthQueryApplicationLineageOutputRole for MergedFaceRole {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryEntityOutputSubject<Face>;
    type Action = WorthQueryMergeOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "merged-face";
}

struct SplitSeamRole;
impl WorthQueryApplicationLineageOutputRole for SplitSeamRole {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryRelationOutputSubject<Seam>;
    type Action = WorthQuerySplitOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "split-seam";
}

struct MergedSeamRole;
impl WorthQueryApplicationLineageOutputRole for MergedSeamRole {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryRelationOutputSubject<Seam>;
    type Action = WorthQueryMergeOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "merged-seam";
}

struct CreatedFaceRole;
impl WorthQueryApplicationLineageOutputRole for CreatedFaceRole {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryEntityOutputSubject<Face>;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "created-face";
}

struct PreservedFaceRole;
impl WorthQueryApplicationLineageOutputRole for PreservedFaceRole {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryEntityOutputSubject<Face>;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "preserved-face";
}

struct RetiredSeamRole;
impl WorthQueryApplicationLineageOutputRole for RetiredSeamRole {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryRelationOutputSubject<Seam>;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "retired-seam";
}

struct SplitFaceFamily;
impl WorthQueryApplicationLineageOutputRoleFamily for SplitFaceFamily {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryEntityOutputSubject<Face>;
    const PREFIX: &'static str = "split-face:";
    const POSTURES: ApplicationMutationLineagePostureSet =
        ApplicationMutationLineagePostureSet::SPLIT
            .union(ApplicationMutationLineagePostureSet::PRESERVE);
    const MINIMUM: usize = 1;
}

struct SplitSeamFamily;
impl WorthQueryApplicationLineageOutputRoleFamily for SplitSeamFamily {
    type Schema = CadSchema;
    type Contract = BooleanSplitMergeOutputs;
    type Subject = WorthQueryRelationOutputSubject<Seam>;
    const PREFIX: &'static str = "split-seam:";
    const POSTURES: ApplicationMutationLineagePostureSet =
        ApplicationMutationLineagePostureSet::ALL;
    const MINIMUM: usize = 0;
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CutDiscriminator {
    Piece {
        cutter: WorthQueryName<Face>,
        side: u8,
    },
    Merged {
        label: String,
    },
}

worth_query_name_discriminator! {
    enum CutDiscriminator {
        identity: "worth.cad.cut_disc.v1",
        variants: {
            Piece { cutter, side },
            Merged { label },
        }
    }
}

#[test]
fn entity_and_relation_lineage_roles_compile_and_expose_uniform_descriptors() {
    const _: () = SplitFacePieceRole::DECLARED;
    const _: () = MergedFaceRole::DECLARED;
    const _: () = SplitSeamRole::DECLARED;
    const _: () = MergedSeamRole::DECLARED;
    const _: () = CreatedFaceRole::DECLARED;
    const _: () = PreservedFaceRole::DECLARED;
    const _: () = RetiredSeamRole::DECLARED;
    const _: () = SplitFaceFamily::DECLARED;
    const _: () = SplitSeamFamily::DECLARED;

    assert_eq!(
        SplitFacePieceRole::DESCRIPTOR.subject_kind(),
        ApplicationMutationLineageSubjectKind::Entity
    );
    assert_eq!(SplitFacePieceRole::DESCRIPTOR.subject(), "Face");
    assert_eq!(
        SplitFacePieceRole::DESCRIPTOR.posture(),
        ApplicationMutationLineagePostureKind::Split
    );

    assert_eq!(
        SplitSeamRole::DESCRIPTOR.subject_kind(),
        ApplicationMutationLineageSubjectKind::Relation
    );
    assert_eq!(SplitSeamRole::DESCRIPTOR.subject(), "Seam");
    assert_eq!(
        SplitSeamRole::DESCRIPTOR.posture(),
        ApplicationMutationLineagePostureKind::Split
    );
    assert_eq!(
        MergedSeamRole::DESCRIPTOR.posture(),
        ApplicationMutationLineagePostureKind::Merge
    );
    assert_eq!(
        SplitSeamFamily::member_name("piece-1").unwrap(),
        "split-seam:piece-1"
    );
}

#[test]
fn lineage_posture_kind_and_bitset_round_trip_cleanly() {
    for kind in [
        ApplicationMutationLineagePostureKind::Preserve,
        ApplicationMutationLineagePostureKind::Create,
        ApplicationMutationLineagePostureKind::Retire,
        ApplicationMutationLineagePostureKind::Split,
        ApplicationMutationLineagePostureKind::Merge,
    ] {
        assert_eq!(
            ApplicationMutationLineagePostureKind::from_canonical_name(kind.canonical_name()),
            Some(kind)
        );
        let single = ApplicationMutationLineagePostureSet::one(kind);
        assert!(single.allows(kind));
        assert!(ApplicationMutationLineagePostureSet::ALL.allows(kind));
        assert_eq!(
            ApplicationMutationLineagePostureSet::from_bits(single.bits()),
            Some(single)
        );
    }

    assert_eq!(
        ApplicationMutationLineagePostureKind::from(ApplicationMutationOutputPosture::Create)
            .try_to_output_posture(),
        Some(ApplicationMutationOutputPosture::Create)
    );
    assert_eq!(
        ApplicationMutationLineagePostureKind::Split.try_to_output_posture(),
        None
    );
    assert_eq!(
        ApplicationMutationLineagePostureKind::Merge.try_to_output_posture(),
        None
    );
    assert_eq!(ApplicationMutationLineagePostureSet::from_bits(0), None);
    assert_eq!(ApplicationMutationLineagePostureSet::from_bits(32), None);
    assert_eq!(
        ApplicationMutationLineagePostureSet::from_output_posture_set(
            ApplicationMutationOutputPostureSet::ALL
        )
        .bits(),
        7
    );
    for subject_kind in [
        ApplicationMutationLineageSubjectKind::Entity,
        ApplicationMutationLineageSubjectKind::Relation,
    ] {
        assert_eq!(
            ApplicationMutationLineageSubjectKind::from_canonical_name(
                subject_kind.canonical_name()
            ),
            Some(subject_kind)
        );
    }
}

#[test]
fn typed_split_and_merge_postures_validate_shape_and_erase_deterministically() {
    let p1 = WorthQueryName::<Face>::from_admitted_digest([0x22; 32]);
    let p2 = WorthQueryName::<Face>::from_admitted_digest([0x11; 32]);
    let outsider = WorthQueryName::<Face>::from_admitted_digest([0x99; 32]);

    let split = WorthQueryLineageOutputPosture::Split {
        parent: p1,
        discriminator: CutDiscriminator::Piece {
            cutter: p2,
            side: 1,
        },
    };
    assert_eq!(split.kind(), ApplicationMutationLineagePostureKind::Split);
    let erased_split = split.to_erased_posture().expect("valid split erases");
    assert_eq!(
        erased_split,
        ErasedLineageOutputPosture::Split {
            parent_digest: [0x22; 32],
            canonical_discriminator: CutDiscriminator::Piece {
                cutter: p2,
                side: 1,
            }
            .canonical_bytes(),
            referenced_name_digests: vec![[0x11; 32]],
        }
    );

    let merge_keep = WorthQueryLineageOutputPosture::<Face, CutDiscriminator>::Merge {
        parents: vec![p1, p2],
        keeper: WorthQueryMergeKeeper::KeepParent(p1),
    };
    let erased_keep = merge_keep
        .to_erased_posture()
        .expect("valid keep-parent merge erases");
    assert_eq!(
        erased_keep,
        ErasedLineageOutputPosture::Merge {
            // Parents canonicalize into ascending byte order.
            parent_digests: vec![[0x11; 32], [0x22; 32]],
            keeper: ErasedMergeKeeper::KeepParent([0x22; 32]),
        }
    );

    let merge_mint = WorthQueryLineageOutputPosture::Merge {
        parents: vec![p1, p2],
        keeper: WorthQueryMergeKeeper::MintNew {
            discriminator: CutDiscriminator::Merged {
                label: "union-top".to_owned(),
            },
        },
    };
    assert!(matches!(
        merge_mint.to_erased_posture(),
        Ok(ErasedLineageOutputPosture::Merge {
            parent_digests,
            keeper: ErasedMergeKeeper::MintNew { .. },
        }) if parent_digests == vec![[0x11; 32], [0x22; 32]]
    ));

    // Invalid merge keeper outside parents refuses typed.
    let bad_keeper = WorthQueryLineageOutputPosture::<Face>::Merge {
        parents: vec![p1, p2],
        keeper: WorthQueryMergeKeeper::KeepParent(outsider),
    };
    assert_eq!(
        bad_keeper.validate_shape(),
        Err(WorthQueryLineagePostureDenial::InvalidMergeKeeper {
            keeper_hex: outsider.render_hex(),
        })
    );

    // Empty, single-parent, and duplicate-parent merges refuse typed.
    let empty_merge = WorthQueryLineageOutputPosture::<Face>::Merge {
        parents: Vec::new(),
        keeper: WorthQueryMergeKeeper::KeepParent(p1),
    };
    assert_eq!(
        empty_merge.validate_shape(),
        Err(WorthQueryLineagePostureDenial::EmptyMergeParents)
    );

    let single_merge = WorthQueryLineageOutputPosture::<Face>::Merge {
        parents: vec![p1],
        keeper: WorthQueryMergeKeeper::KeepParent(p1),
    };
    assert_eq!(
        single_merge.validate_shape(),
        Err(WorthQueryLineagePostureDenial::InsufficientMergeParents { count: 1 })
    );

    let dup_merge = WorthQueryLineageOutputPosture::<Face>::Merge {
        parents: vec![p1, p1],
        keeper: WorthQueryMergeKeeper::KeepParent(p1),
    };
    assert_eq!(
        dup_merge.validate_shape(),
        Err(WorthQueryLineagePostureDenial::DuplicateMergeParent {
            parent_hex: p1.render_hex(),
        })
    );
}
