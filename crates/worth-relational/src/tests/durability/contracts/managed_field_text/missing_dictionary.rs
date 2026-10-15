//! A real current tail keeps its symbolic payload while its required meaning is removed.
use super::*;

#[test]
fn native_field_text_missing_tail_dictionary_refuses_without_target_or_source_effect() {
    let lease = test_execution_lease();
    let execution = ExecutionRequest::leased(&lease);
    let source = persisted_runtime_with_test_schema();
    let baseline = create_entity_outcome(&execution, &source, "before-missing-meaning");
    release_test_commit_snapshot(&source, &baseline);
    let config = source.config();
    source
        .durability_authority()
        .checkpoint(&execution)
        .unwrap();
    let committed = commit_text(&execution, &source).unwrap();
    release_test_commit_snapshot(&source, &committed);
    let path = source.durable_store().unwrap().segments[0].path.clone();
    let mut file =
        crate::durability::log::native_file_codec::read_segment_file(&execution, &path).unwrap();
    let tail = file.entries.last_mut().unwrap().envelope_mut_for_test();
    assert!(
        crate::symbols::data::StringInterner::default()
            .commit_texts(&tail.merged_plan, &tail.patch)
            .is_err(),
        "producer must refuse unresolved references, never omit their rows"
    );
    let required = crate::symbols::data::required_commit_symbols(&tail.merged_plan, &tail.patch);
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
    // Existing current-format fixture writer preserves the actual plan/patch and execution.
    crate::durability::log::native_file_codec::write_segment_file(&execution, &path, &file)
        .unwrap();
    let malformed = std::fs::read(&path).unwrap();
    let mut target = crate::runtime::RelationalRuntime::new(config);
    let identity = target.main_branch_identity();
    let before = target.observe_branch(&identity).unwrap().0;
    let symbols = target.services.symbols.snapshot();
    let plan = target
        .durability()
        .recovery_plan(
            &execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let error = target
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap_err();
    assert!(
        matches!(error, RelationalRecoveryStop::Durability(ref error)
        if error.class == RecoveryFailureClass::CorruptCheckpoint)
    );
    assert_eq!(target.observe_branch(&identity).unwrap().0, before);
    assert_eq!(target.services.symbols.snapshot(), symbols);
    assert_eq!(std::fs::read(&path).unwrap(), malformed);
}
