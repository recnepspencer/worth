//! Reconstruction capabilities are minted only at exact trust boundaries.
use crate::{
    config::{diagnostic, Code, Diagnostic, CONFIG},
    inputs::Inputs,
    lexical::tokens,
};
use std::collections::BTreeSet;

const CONSTRUCTOR: &str = "enter_reconstruction";
const DEFINITION: &str =
    "crates/worth-relational/src/storage/substrate/maintenance/reconstruction.rs";

pub(super) fn check(input: &Inputs) -> Vec<Diagnostic> {
    let references: Vec<_> = input
        .files
        .iter()
        .filter(|(path, source)| {
            path.ends_with(".rs")
                && source.contains(CONSTRUCTOR)
                && tokens(source).iter().any(|token| token.text == CONSTRUCTOR)
        })
        .map(|(path, _)| path.as_str())
        .collect();
    if references.is_empty() {
        return Vec::new();
    }
    let registration = format!("{CONFIG}reconstruction-callers.txt");
    let Some(policy) = input.files.get(&registration) else {
        return vec![diagnostic(
            Code::InputFailure,
            &registration,
            "missing reconstruction caller policy",
        )];
    };
    let callers: BTreeSet<_> = policy
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let mut out = Vec::new();
    for caller in &callers {
        if !input.paths.contains(*caller) {
            out.push(diagnostic(
                Code::InputFailure,
                &registration,
                format!("registered reconstruction caller does not exist: {caller}"),
            ));
        }
    }
    for path in references {
        if path != DEFINITION && !callers.contains(path) {
            out.push(diagnostic(Code::Reconstruction, path,
                "reconstruction constructor requires an exact cold restore, initialization, checkpoint or test-oracle caller"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const ORDINARY: &str = "crates/worth-relational/src/storage/residency/region_publication.rs";
    const COLD: &str = "crates/worth-relational/src/storage/residency/region_load.rs";

    fn fixture(path: &str, source: &str) -> Inputs {
        Inputs::fixture(
            &[
                (DEFINITION, "pub fn enter_reconstruction() {}"),
                (COLD, if path == COLD { source } else { "" }),
                (ORDINARY, if path == ORDINARY { source } else { "" }),
                (
                    "tools/boundary-check/config/reconstruction-callers.txt",
                    COLD,
                ),
            ],
            "",
        )
    }
    #[test]
    fn ordinary_commit_callers_cannot_mint_a_reconstruction_lane() {
        for source in [
            "ReconstructionLane::enter_reconstruction(|lane| cold(lane));",
            "use sdk::ReconstructionLane as Lane; Lane::enter_reconstruction(|_| ());",
            "let constructor = sdk::ReconstructionLane::r#enter_reconstruction;",
            "use sdk::enter_reconstruction as ordinary;",
        ] {
            let errors = check(&fixture(ORDINARY, source));
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == Code::Reconstruction && error.subject == ORDINARY),
                "{source}"
            );
        }
    }
    #[test]
    fn composite_reconstruction_helpers_are_not_constructor_boundaries() {
        for helper in [
            "region_initialization",
            "checkpoint_kind_images",
            "region_capture",
        ] {
            let path = format!("crates/worth-relational/src/storage/residency/{helper}.rs");
            let input = Inputs::fixture(
                &[
                    (DEFINITION, "pub fn enter_reconstruction() {}"),
                    (COLD, ""),
                    (&path, "ReconstructionLane::enter_reconstruction(|_| ());"),
                    (
                        "tools/boundary-check/config/reconstruction-callers.txt",
                        include_str!("../../config/reconstruction-callers.txt"),
                    ),
                ],
                "",
            );
            assert!(check(&input)
                .iter()
                .any(|error| error.code == Code::Reconstruction && error.subject == path));
        }
    }
    #[test]
    fn exact_trust_boundaries_can_mint_without_granting_their_crate() {
        assert!(check(&fixture(
            COLD,
            "ReconstructionLane::enter_reconstruction(|_| ());"
        ))
        .is_empty());
        assert!(check(&fixture(
            ORDINARY,
            "// enter_reconstruction\n let description = \"enter_reconstruction\";"
        ))
        .is_empty());
    }
}
