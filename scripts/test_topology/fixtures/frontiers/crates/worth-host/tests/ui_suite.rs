#[path = "ui_support.rs"]
mod boundary_inputs;
mod compile_fail {
    #[test]
    fn ui_program_can_name_its_original_dev_dependency() {
        trybuild::TestCases::new().pass(crate::boundary_inputs::INPUT);
    }
}
