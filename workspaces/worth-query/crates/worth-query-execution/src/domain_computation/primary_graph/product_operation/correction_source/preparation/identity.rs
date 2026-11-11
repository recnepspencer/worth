//! Prepared correction identity preserves the exact original and target meaning.
use super::*;

impl PartialEq for WorthQueryPreparedCorrectionSource {
    fn eq(&self, other: &Self) -> bool {
        self.runtime == other.runtime
            && self.authored_identity == other.authored_identity
            && self.workflow == other.workflow
            && self.required_root == other.required_root
            && self.target_program.same_support_as(&other.target_program)
            && self.actor == other.actor
            && self.scope == other.scope
            && self.action_input == other.action_input
            && self.action_binding == other.action_binding
            && self.lineage == other.lineage
            && self.intent == other.intent
            && self.target_parent == other.target_parent
            && self.target_branch == other.target_branch
            && self.source_world() == other.source_world()
            && self.source_commit() == other.source_commit()
            && self.target == other.target
    }
}
impl Eq for WorthQueryPreparedCorrectionSource {}
