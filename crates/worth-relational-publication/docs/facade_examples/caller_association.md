```compile_fail,E0599
use worth_relational::facade::mvcc::{PreparedRelationalCommitCandidate, RelationalCallerAssociation};
fn cannot_rebind(candidate: PreparedRelationalCommitCandidate, association: RelationalCallerAssociation) {
    candidate.bind_caller_association(association);
}
```
