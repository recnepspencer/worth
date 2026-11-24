```compile_fail
use worth_relational::facade::transactions::CommitResult;

fn cannot_swap_outcomes(target: &mut CommitResult, other: &CommitResult) {
    target.outcome = other.outcome().clone();
}
```
