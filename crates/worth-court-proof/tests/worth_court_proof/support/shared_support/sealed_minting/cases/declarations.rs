pub(crate) use {std::collections::BTreeSet};
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {crate::shared_support::compile_fail::bundle::CompileFailBundle, crate::shared_support::compile_fail::bundle::CompileFailCase};

pub(crate) const SUITE: &str = "sealed_minting_and_witness_authority";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SealedMintingFamily {
    ProofMinting,
    ProofAuthority,
    WitnessMinting,
    WitnessBoundaries,
    RecipeBoundaries,
}

impl SealedMintingFamily {
    pub(crate) const fn label(self) -> &'static str {
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
pub(crate) enum SealedMintingObligation {
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
pub(crate) struct SealedMintingCase {
    pub(crate) family: SealedMintingFamily,
    pub(crate) obligation: SealedMintingObligation,
    pub(crate) path: &'static str,
}

impl SealedMintingCase {
    pub(crate) const fn new(
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

    pub(crate) const fn family(self) -> &'static str {
        self.family.label()
    }

    pub(crate) const fn obligation(self) -> SealedMintingObligation {
        self.obligation
    }

    pub(crate) const fn path(self) -> &'static str {
        self.path
    }
}

pub(crate) const CASES: &[SealedMintingCase] = &[
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::ProofMinting,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::StrongerConstructorPrivate,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/stronger_proof_bearing_constructors_are_not_public.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::ProofMinting,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::ObservedProofMoveOnly,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/observed_proofs_cannot_be_duplicated.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::ProofAuthority,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::ProofAuthorityScopeExact,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/proof_authority_scope_cannot_be_substituted.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::ProofAuthority,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::UnprovenProofKindDenied,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/authority_cannot_mint_unproven_proof_kind.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::ProofAuthority,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::MixedAuthorityProofSetDenied,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/current_basis_rejects_mixed_authority_proof_set.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::WitnessMinting,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::WitnessMintPrivate,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/witnesses_are_not_publicly_mintable.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::WitnessMinting,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::MarkerMintPrivate,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/sealed_markers_are_not_mintable_by_consumers.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::WitnessBoundaries,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::WitnessRequired,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/witness_required_apis_reject_callers_without_witness.rs"),
    ),
    crate::shared_support::sealed_minting::cases::SealedMintingCase::new(
        crate::shared_support::sealed_minting::cases::SealedMintingFamily::RecipeBoundaries,
        crate::shared_support::sealed_minting::cases::SealedMintingObligation::RecipeStagesOrdered,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting/recipe_stages_are_not_publicly_skippable.rs"),
    ),
];

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::shared_support::compile_fail::bundle::CompileFailBundle::new(
        SUITE,
        CASES
            .iter()
            .map(|case| crate::shared_support::compile_fail::bundle::CompileFailCase::new(case.family(), case.path()))
            .collect(),
    )
}

pub(crate) fn assert_fixture_completeness() {
    let root = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-proof"));
    let discovered = fixture_paths(&root.join(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/sealed_minting")), &root);
    let catalog = CASES
        .iter()
        .map(|case| case.path().replace('\\', "/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(discovered, catalog, "sealed-minting fixture catalog drift");
}

pub(crate) fn fixture_paths(ui_root: &Path, crate_root: &Path) -> BTreeSet<String> {
    let mut pending = vec![ui_root.to_path_buf()];
    let mut discovered = std::collections::BTreeSet::new();
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