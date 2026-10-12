//! The authoritative intent declaration used by certification fixtures.
use super::*;

pub(in crate::intent_admission::certification) fn authoritative_declaration(
    name: &str,
) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        name,
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        crate::runtime::WorthQueryIntentInput::object([
            (
                "entity",
                crate::runtime::WorthQueryIntentInput::string("task-1"),
            ),
            (
                "title",
                crate::runtime::WorthQueryIntentInput::string("Certification committed title"),
            ),
        ]),
    )
}
