```compile_fail,E0451
use worth_relational::facade::mvcc::RelationalPredecessorInput;
fn forge() -> RelationalPredecessorInput {
    RelationalPredecessorInput { before: panic!(), applied: None }
}
```
