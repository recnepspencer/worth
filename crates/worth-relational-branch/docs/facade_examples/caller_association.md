```compile_fail,E0423
use worth_relational::facade::mvcc::RelationalCallerAssociation;
let fabricated = RelationalCallerAssociation(std::sync::Arc::from(&b"caller"[..]));
```
