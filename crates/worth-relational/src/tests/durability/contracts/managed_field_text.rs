//! Real native payload custody through current checkpoint and post-checkpoint tail.
use super::*;

use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

const JSON: &str = r#"{"actor":"native","request":{"input":"exact logical content"}}"#;

#[test]
fn native_field_text_configuration_requires_current_explicit_readmission() {
    let lease = test_execution_lease();
    let execution = &ExecutionRequest::leased(&lease);
    let source = runtime_with_test_schema();
    let (_, basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(JSON.as_bytes().into()));
    let field = source.admit_field_text(execution, &basis, &input).unwrap();
    let snapshot = source.config().identity.symbol_table;
    let wire = rmp_serde::to_vec_named(&snapshot).unwrap();
    assert!(rmp_serde::from_slice::<crate::symbols::data::SymbolTableSnapshot>(&wire).is_err());
    drop((snapshot, field, source));
    let admitted =
        crate::symbols::data::SymbolTableSnapshot::readmit_current(execution, &wire).unwrap();
    let managed = admitted
        .entries
        .iter()
        .find(|(_, text)| text.as_str() == JSON)
        .unwrap();
    assert_eq!(managed.1.charged_payload_bytes(), Some(JSON.len() as u64));
    let mut config = runtime_with_test_schema().config();
    config.identity.symbol_table = admitted;
    let restored =
        crate::runtime::RelationalRuntime::new(config).expect("valid native runtime configuration");
    restored.services.symbols.with_read(|symbols| {
        assert_eq!(symbols.resolve(symbols.symbol(JSON).unwrap()), Some(JSON))
    });
}
