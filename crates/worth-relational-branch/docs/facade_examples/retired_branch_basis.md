```compile_fail
use worth_relational::facade::branch::RetiredRelationalBranchBasis;
fn cannot_read(value: RetiredRelationalBranchBasis) { value.observation(); }
```
