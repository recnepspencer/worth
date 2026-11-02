//! Running a script against a durable home and comparing it with the
//! reference.

pub(crate) use std::path::Path;

pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use {
    worth_store_application::ApplicationHomeRequest,
    worth_store_application::ReadmittedApplicationHome,
};

pub(crate) use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused;

pub(crate) use worth_relational::facade::schema::SchemaVersionId;
pub(crate) use {
    worth_relational::facade::durability::RecoveryAuthorityContinuityMismatch,
    worth_relational::facade::durability::RecoveryFailureClass,
    worth_relational::facade::durability::RelationalRecoveryStop,
};

pub(crate) use crate::schema_transition;
pub(crate) use {crate::workload, crate::workload::Script, crate::workload::Step};

/// Open the home declaring the schema it serves (D5): the script's source
/// schema until its durable transition, the target after. A crash can leave
/// the transition's fate to recovery, so when the home refuses the source
/// because its recovered main head records the target, open declaring that.
pub(crate) fn open(
    location: &Path,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
    match open_declaring(location, crate::schema_transition::SOURCE, execution) {
        Err(refusal) if recorded_schema(&refusal) == Some(crate::schema_transition::TARGET) => {
            // The original refused Home still owns the physical lease. Settle
            // that actual raw owner before requesting a differently declared one.
            match refusal {
                HomeOpeningRefused::Readmission(
                    worth_store_application::RelationalHomeOpenRefused::Readmission(refused),
                ) => match refused.into_home().close(execution) {
                    Ok(worth_store_application::ApplicationHomeCloseOutcome::Closed { .. }) => {
                        open_declaring(location, crate::schema_transition::TARGET, execution)
                    }
                    Ok(outcome) => Err(HomeOpeningRefused::Close(outcome)),
                    Err(home) => Err(HomeOpeningRefused::UnrecoveredClose(home)),
                },
                other => Err(other),
            }
        }
        opened => opened,
    }
}

/// The schema a refused open's recovered main head records, when the refusal
/// is that the declared schema differs from it.
pub(crate) fn recorded_schema(refusal: &HomeOpeningRefused) -> Option<SchemaVersionId> {
    let Some(worth_store_application::ApplicationHomeOpenRefusal::Relational(
        worth_store_relational::RelationalSectionReadmissionDenial::Native(
            worth_relational::facade::durability::RelationalRecoveryStop::Durability(error),
        ),
    )) = refusal.source_cause()
    else {
        return None;
    };
    match (&error.class, &error.authority_continuity_mismatch) {
        (
            worth_relational::facade::durability::RecoveryFailureClass::DeclaredSchemaMismatch,
            Some(worth_relational::facade::durability::RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
                expected_primary_schema_version,
                ..
            }),
        ) => Some(*expected_primary_schema_version),
        _ => None,
    }
}

/// The schema a cleanly closed home holding `steps` serves: the transition's
/// target once the script's transition is among them, the source before.
pub(crate) fn served_schema(steps: &[Step]) -> SchemaVersionId {
    if steps
        .iter()
        .any(|step| matches!(step, crate::workload::Step::Transition { .. }))
    {
        crate::schema_transition::TARGET
    } else {
        crate::schema_transition::SOURCE
    }
}

/// Open the home with a runtime that declares the schema registry at `schema`.
pub(crate) fn open_declaring(
    location: &Path,
    schema: SchemaVersionId,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
    worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(
            location,
            crate::workload::runtime_builder(schema),
        ),
        execution,
    )
}

/// Apply `steps` to the home, returning how many commits were acknowledged.
pub(crate) fn apply(
    home: &ReadmittedApplicationHome,
    execution: &ExecutionRequest<'_, '_>,
    steps: &[Step],
) -> usize {
    let mut acknowledged = 0;
    for step in steps {
        match step {
            crate::workload::Step::Commit { .. } | crate::workload::Step::Transition { .. } => {
                let completed = step
                    .commit(home.relational().runtime(), execution)
                    .unwrap_or_else(|error| panic!("durable commit {acknowledged}: {error:?}"));
                // The script consumes the result here; only explicit snapshot
                // cases retain a read pin beyond an acknowledged operation.
                home.relational()
                    .runtime()
                    .snapshots()
                    .release_snapshot(&completed.snapshot)
                    .unwrap();
                acknowledged += 1;
            }
            crate::workload::Step::Checkpoint => {
                home.checkpoint(execution).expect("home checkpoint");
            }
            _ => step
                .lifecycle(home.relational().runtime(), execution)
                .expect("durable lifecycle"),
        }
    }
    acknowledged
}

/// Require the home to hold exactly the commits the durable run acknowledged
/// (`acknowledged`), each byte-identical to the reference's artifact at its
/// position (D11), and the reference's record states after that many commits.
pub(crate) fn assert_holds(
    execution: &ExecutionRequest<'_, '_>,
    home: &ReadmittedApplicationHome,
    script: &Script,
    acknowledged: &[Vec<u8>],
) {
    let artifacts = crate::workload::artifacts_of(execution, home.relational().runtime());
    let commits = acknowledged.len();
    assert_eq!(artifacts.len(), commits, "recovered commit count");
    for (index, (found, expected)) in artifacts.iter().zip(acknowledged).enumerate() {
        assert!(
            found == expected,
            "artifact at position {} differs from the acknowledged one",
            index + 1
        );
    }
    assert_reference_artifacts(&artifacts, script, "held");
    assert_lifecycle_prefix(home, script, commits, "held");
}

pub(crate) fn assert_lifecycle_prefix(
    home: &ReadmittedApplicationHome,
    script: &Script,
    commits: usize,
    case: &str,
) {
    let transitions = crate::branch_lifecycle::artifacts_of_home(home);
    assert_eq!(
        transitions,
        script.transitions[..transitions.len()],
        "{case}: transition artifacts"
    );
    let expected = script
        .prefixes
        .get(&(commits, transitions.len()))
        .expect("a legal reference prefix");
    assert_eq!(
        &crate::branch_lifecycle::state_of(home.relational().runtime(), &script.records),
        expected,
        "{case}: records, branch existence and lifecycle"
    );
    assert_eq!(
        &crate::branch_lifecycle::state_of(home.relational().runtime(), &script.records),
        expected,
        "{case}: warm records, branch existence and lifecycle"
    );
}

/// Require every artifact to equal the reference's at its position.
pub(crate) fn assert_reference_artifacts(artifacts: &[Vec<u8>], script: &Script, case: &str) {
    assert!(
        artifacts.len() <= script.artifacts.len(),
        "{case}: more artifacts than the reference"
    );
    for (index, (found, reference)) in artifacts.iter().zip(&script.artifacts).enumerate() {
        assert!(
            found == reference,
            "{case}: artifact at position {} differs from the reference's",
            index + 1
        );
    }
}

/// The steps that follow the `commits`-th commit (and any checkpoint after it).
pub(crate) fn steps_after(script: &Script, commits: usize) -> &[Step] {
    let mut seen = 0;
    for (index, step) in script.steps.iter().enumerate() {
        if seen == commits && !matches!(step, crate::workload::Step::Checkpoint) {
            return &script.steps[index..];
        }
        if step.is_commit() {
            seen += 1;
        }
    }
    &[]
}

/// The steps up to and including the `commits`-th commit.
pub(crate) fn steps_before(script: &Script, commits: usize) -> &[Step] {
    let rest = steps_after(script, commits).len();
    &script.steps[..script.steps.len() - rest]
}
