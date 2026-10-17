use worth_execution::ChargedBytes;

#[derive(ChargedBytes)]
struct Generic<T>(T);
#[derive(ChargedBytes)]
union Ambiguous {
    integer: u64,
    floating: f64,
}
fn main() {}
