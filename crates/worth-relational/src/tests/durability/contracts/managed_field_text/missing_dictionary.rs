//! A durable tail artifact keeps its symbolic payload while its required
//! meaning is removed.
use super::*;

#[test]
fn native_field_text_missing_tail_dictionary_refuses_without_target_or_source_effect() {
    let lease = test_execution_lease();
    let execution = ExecutionRequest::leased(&lease);
    let source = durable_runtime_with_test_schema();
    let baseline = create_entity_outcome(&execution, &source, "before-missing-meaning");
    release_test_commit_snapshot(&source, &baseline);
    let config = source.config();
    crate::tests::support::checkpoint_durably(&execution, &source).unwrap();
    let committed = commit_text(&execution, &source).unwrap();
    release_test_commit_snapshot(&source, &committed);
    let medium = attached_test_medium(&source);
    let tail_position = medium.head();
    medium.rewrite_artifact_envelope(&execution, tail_position, |tail| {
        assert!(
            crate::symbols::data::StringInterner::default()
                .commit_texts(&tail.merged_plan, &tail.patch)
                .is_err(),
            "producer must refuse unresolved references, never omit their rows"
        );
        let required =
            crate::symbols::data::required_commit_symbols(&tail.merged_plan, &tail.patch);
        let managed = tail
            .symbol_texts
            .entries
            .iter()
            .find(|(_, text)| text.as_str() == JSON)
            .unwrap()
            .0;
        assert!(required.contains(&managed));
        tail.symbol_texts
            .entries
            .retain(|(symbol, _)| *symbol != managed);
        assert_eq!(
            crate::symbols::data::required_commit_symbols(&tail.merged_plan, &tail.patch),
            required
        );
    });
    let malformed = medium.artifact_bytes(tail_position).unwrap();
    let mut target = crate::runtime::RelationalRuntime::new(config);
    bind_durable_medium(&mut target, medium.shared());
    let identity = target.main_branch_identity();
    let before = target.observe_branch(&identity).unwrap().0;
    let symbols = target.services.symbols.snapshot();
    let error = target
        .durability()
        .recovery_plan_from_source(
            &execution,
            medium.recovery_source().unwrap(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap_err();
    assert!(
        matches!(error, RelationalRecoveryStop::Durability(ref error)
        if error.class == RecoveryFailureClass::CorruptCanonicalArtifact)
    );
    assert_eq!(target.observe_branch(&identity).unwrap().0, before);
    assert_eq!(target.services.symbols.snapshot(), symbols);
    assert_eq!(medium.artifact_bytes(tail_position).unwrap(), malformed);
}
