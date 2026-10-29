use worth_foundational::{AuthoritativeRecordAspectPatch, AuthoritativeRecordAspectState};

fn requires_state(_state: &AuthoritativeRecordAspectState) {}

fn main() {
    let patch = AuthoritativeRecordAspectPatch::empty();
    requires_state(&patch);
}
