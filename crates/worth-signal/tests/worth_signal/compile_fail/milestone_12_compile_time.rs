#[test]
fn invalidation_authority_forms_are_not_publicly_mintable() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/milestone_12/*.rs");
}
