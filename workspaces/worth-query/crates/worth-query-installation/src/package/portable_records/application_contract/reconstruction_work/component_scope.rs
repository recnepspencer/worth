//! Recursive scope metadata in the existing untrusted package reconstruction lane.
use super::WorthQueryPortableApplicationContractReconstructionWork;
use worth_query_declaration::facade::application_schema::ApplicationOperationComponentReadMeaning;

impl WorthQueryPortableApplicationContractReconstructionWork {
    pub(super) fn observe_component_scope(
        &mut self,
        meaning: &ApplicationOperationComponentReadMeaning,
    ) {
        for text in [
            meaning.operation(),
            meaning.set(),
            meaning.root(),
            meaning.member(),
        ] {
            self.observe_text(text);
        }
        self.observe_fixed(4);
        for path in meaning.paths() {
            self.observe_text(path.start_entity());
            self.observe_text(path.terminal_entity());
            self.observe_fixed(8);
            for step in path.steps() {
                for text in [step.relation(), step.from(), step.to()] {
                    self.observe_text(text);
                }
                self.observe_fixed(2);
            }
            for guard in path.guards() {
                for text in [
                    guard.entity(),
                    guard.aspect(),
                    guard.field(),
                    guard.value_type(),
                ] {
                    self.observe_text(text);
                }
                self.observe_fixed(10);
                self.observe_fixed(
                    u64::try_from(guard.expected().semantic_byte_width()).unwrap_or(u64::MAX),
                );
            }
        }
    }
}
