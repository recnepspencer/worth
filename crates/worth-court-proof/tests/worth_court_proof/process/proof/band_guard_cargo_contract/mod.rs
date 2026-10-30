use crate::band_guard_cargo_contract::process_context::*;

#[test]
fn macro_arguments_select_disjoint_expanding_package_bands() {
    let observations = PrebuiltProofObservations::from_environment().unwrap();

    for legal_package in [
        "worth-entry-consumer",
        "worthy-entry-consumer",
        "worth-cert-consumer",
    ] {
        let observed = observations.compiler_observation(legal_package).unwrap();
        assert_eq!(
            observed.outcome,
            PackageCompilation::Compiled,
            "legal fixture {legal_package} failed:\n{}",
            observed.output()
        );
    }
    assert_package_is_denied(
        "worth-schema-consumer",
        &["worth-entry-", "worthy-entry-"],
        &observations,
    );
    assert_package_is_denied(
        "worth-entry-cert-consumer",
        &["worth-cert-", "worthy-cert-"],
        &observations,
    );
}

#[test]
fn empty_prefix_cannot_disable_the_expansion_fence() {
    let observations = PrebuiltProofObservations::from_environment().unwrap();
    let observed = observations
        .compiler_observation("empty-prefix-consumer")
        .unwrap();
    let output_text = observed.output();
    assert!(
        observed.outcome == PackageCompilation::Denied,
        "empty prefix unexpectedly compiled"
    );
    assert!(
        output_text.contains("worth_proof::band_guard! rejected package"),
        "empty-prefix denial omitted the band-guard diagnostic:\n{output_text}"
    );
}

#[test]
fn production_package_has_no_normal_dependencies() {
    let observations = PrebuiltProofObservations::from_environment().unwrap();
    let observed = observations.normal_dependencies().unwrap();
    assert_eq!(
        observed.exit_code,
        0,
        "cargo tree failed:\n{}",
        observed.output()
    );
    let resolved_packages: Vec<_> = observed
        .stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect();
    assert_eq!(
        resolved_packages.len(),
        1,
        "worth-proof gained a normal dependency: {resolved_packages:#?}"
    );
    assert!(resolved_packages[0].starts_with("worth-proof v"));
}

#[test]
fn successful_guard_expansion_adds_no_runtime_artifact_surface() {
    let observations = PrebuiltProofObservations::from_environment().unwrap();
    let baseline_surface = observations
        .runtime_surface(RuntimeSpecimen::Baseline)
        .unwrap();
    let guarded_surface = observations
        .runtime_surface(RuntimeSpecimen::Guarded)
        .unwrap();
    assert!(
        baseline_surface
            .iter()
            .any(|definition| definition.contains("observed_runtime_surface")),
        "artifact inspection missed the specimen's exported runtime function: {baseline_surface:#?}"
    );
    assert_eq!(
        guarded_surface, baseline_surface,
        "a successful band guard expansion changed release LLVM runtime definitions"
    );
}

#[test]
fn missing_prebuilt_observations_are_refused() {
    let missing = std::env::temp_dir().join(format!(
        "worth-proof-missing-prerequisite-{}",
        std::process::id()
    ));
    let result = PrebuiltProofObservations::from_directory(missing);
    assert!(matches!(
        result,
        Err(ProofPrebuildFailure::MissingObservation(_))
    ));
}

fn assert_package_is_denied(
    package: &str,
    legal_prefixes: &[&str],
    observations: &PrebuiltProofObservations,
) {
    let observed = observations.compiler_observation(package).unwrap();
    let output_text = observed.output();
    assert!(
        observed.outcome == PackageCompilation::Denied,
        "wrong-band fixture {package} unexpectedly compiled"
    );
    let legal_list = format!("legal package prefixes: {}", legal_prefixes.join(", "));
    for required_text in [
        legal_list.as_str(),
        "cad/docs/worthy-foundations/BOUNDARIES.md",
        package,
    ] {
        assert!(
            output_text.contains(required_text),
            "diagnostic for {package} omitted {required_text:?}:\n{output_text}"
        );
    }
}
