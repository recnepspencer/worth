use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::compile_fail::{CompileFailBundle, CompileFailCase};

pub const SUITE: &str = "sealed_minting_and_witness_authority";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SealedMintingFamily {
    ProofMinting,
    ProofAuthority,
    WitnessMinting,
    WitnessBoundaries,
    RecipeBoundaries,
}

impl SealedMintingFamily {
    const fn label(self) -> &'static str {
        match self {
            Self::ProofMinting => "sealed_minting",
            Self::ProofAuthority => "proof_authority",
            Self::WitnessMinting => "witness_minting",
            Self::WitnessBoundaries => "witness_boundaries",
            Self::RecipeBoundaries => "recipe_boundaries",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SealedMintingObligation {
    StrongerConstructorPrivate,
    ObservedProofMoveOnly,
    ProofAuthorityScopeExact,
    UnprovenProofKindDenied,
    MixedAuthorityProofSetDenied,
    WitnessMintPrivate,
    MarkerMintPrivate,
    WitnessRequired,
    RecipeStagesOrdered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SealedMintingCase {
    family: SealedMintingFamily,
    obligation: SealedMintingObligation,
    path: &'static str,
}

impl SealedMintingCase {
    pub const fn new(
        family: SealedMintingFamily,
        obligation: SealedMintingObligation,
        path: &'static str,
    ) -> Self {
        Self {
            family,
            obligation,
            path,
        }
    }

    pub const fn family(self) -> &'static str {
        self.family.label()
    }

    pub const fn obligation(self) -> SealedMintingObligation {
        self.obligation
    }

    pub const fn path(self) -> &'static str {
        self.path
    }
}

pub const CASES: &[SealedMintingCase] = &[
    SealedMintingCase::new(
        SealedMintingFamily::ProofMinting,
        SealedMintingObligation::StrongerConstructorPrivate,
        "tests/ui/sealed_minting/stronger_proof_bearing_constructors_are_not_public.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::ProofMinting,
        SealedMintingObligation::ObservedProofMoveOnly,
        "tests/ui/sealed_minting/observed_proofs_cannot_be_duplicated.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::ProofAuthority,
        SealedMintingObligation::ProofAuthorityScopeExact,
        "tests/ui/sealed_minting/proof_authority_scope_cannot_be_substituted.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::ProofAuthority,
        SealedMintingObligation::UnprovenProofKindDenied,
        "tests/ui/sealed_minting/authority_cannot_mint_unproven_proof_kind.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::ProofAuthority,
        SealedMintingObligation::MixedAuthorityProofSetDenied,
        "tests/ui/sealed_minting/current_basis_rejects_mixed_authority_proof_set.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::WitnessMinting,
        SealedMintingObligation::WitnessMintPrivate,
        "tests/ui/sealed_minting/witnesses_are_not_publicly_mintable.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::WitnessMinting,
        SealedMintingObligation::MarkerMintPrivate,
        "tests/ui/sealed_minting/sealed_markers_are_not_mintable_by_consumers.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::WitnessBoundaries,
        SealedMintingObligation::WitnessRequired,
        "tests/ui/sealed_minting/witness_required_apis_reject_callers_without_witness.rs",
    ),
    SealedMintingCase::new(
        SealedMintingFamily::RecipeBoundaries,
        SealedMintingObligation::RecipeStagesOrdered,
        "tests/ui/sealed_minting/recipe_stages_are_not_publicly_skippable.rs",
    ),
];

pub fn compile_fail_bundle() -> CompileFailBundle {
    CompileFailBundle::new(
        SUITE,
        CASES
            .iter()
            .map(|case| CompileFailCase::new(case.family(), case.path()))
            .collect(),
    )
}

pub fn assert_fixture_completeness() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let discovered = fixture_paths(&root.join("tests/ui/sealed_minting"), &root);
    let catalog = CASES
        .iter()
        .map(|case| case.path().replace('\\', "/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(discovered, catalog, "sealed-minting fixture catalog drift");
}

fn fixture_paths(ui_root: &Path, crate_root: &Path) -> BTreeSet<String> {
    let mut pending = vec![ui_root.to_path_buf()];
    let mut discovered = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("sealed-minting fixture directory") {
            let path = entry.expect("sealed-minting fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                discovered.insert(
                    path.strip_prefix(crate_root)
                        .expect("fixture belongs to worth-proof")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    discovered
}
