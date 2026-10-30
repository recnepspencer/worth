use worth_execution::ChargedBytes;

#[derive(ChargedBytes)]
struct Skipped {
    #[charged_bytes(skip)]
    resident: Vec<String>,
}
#[derive(ChargedBytes)]
#[charged_bytes(wire_size)]
struct WireProxy(String);
fn main() {}
