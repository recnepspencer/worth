# Authority boundary compile-fail proofs

Model actions cannot satisfy a production execution boundary:

```compile_fail
use worth_store_formal_models::CompactionVisibilityAction;
use worth_store_physical_isolation::CompactionCutoverDelta;

let action = CompactionVisibilityAction::LowerRewrite;
let _ = CompactionCutoverDelta::lower(action, panic!("root is irrelevant to this type proof"));
```

An owner case id cannot be copied into an owner-issued observation:

```compile_fail
use worth_store_physical_isolation::{
    CompactionOwnerCaseId, CompactionOwnerCaseObservation,
};

let _ = CompactionOwnerCaseObservation {
    declaration: CompactionOwnerCaseId::LowerRewrite,
};
```

Even a genuine read-only observation is not execution authority:

```compile_fail
use worth_store_physical_isolation::{
    CompactionCutoverDelta, CompactionOwnerCaseObservation,
};

fn observation_cannot_execute(observation: CompactionOwnerCaseObservation) {
    let _ = CompactionCutoverDelta::lower(
        observation,
        panic!("root is irrelevant to this type proof"),
    );
}
```

A successful or failed checker verdict cannot become replication publication
authority. This fails on the concrete production-authority type, after every
crate and input type has resolved:

```compile_fail
use worth_store_formal_models::runner::ProtocolCheckVerdict;
use worth_store_replication::{
    ReplicationAdmissionRuntime, ReplicationPublicationReadiness,
};

fn verdict_cannot_publish(
    runtime: &mut ReplicationAdmissionRuntime,
    readiness: ReplicationPublicationReadiness,
    verdict: &ProtocolCheckVerdict,
) {
    let _ = runtime.publish(readiness, verdict);
}
```
