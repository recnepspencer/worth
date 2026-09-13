Store certification vocabulary.

S.2 readiness is minted only by the readiness owner from admitted S.1 evidence:

```compile_fail
use worth_store_readiness::S2PhysicalSubstrateReadiness;
use worth_store_contracts::ROADMAP_2_S1_SCOPE;

let _forged = S2PhysicalSubstrateReadiness {
    scope: ROADMAP_2_S1_SCOPE,
    facts: todo!(),
    sealed: true,
};
```

S.6 production readiness closure is owned by `worth_store_readiness`, not certification:

```compile_fail
use worth_store_certification::close_s6_production_readiness;
```

S.6 IO/QoS readiness is minted by `worth_store_physical_isolation`, not certification:

```compile_fail
use worth_store_certification::materialize_s6_io_qos_isolation_readiness;
```

S.3 physical integrity readiness cannot be minted from raw payload fields:

```compile_fail
use worth_store_readiness::S3PhysicalIntegrityReadiness;

let _forged = S3PhysicalIntegrityReadiness {
    s2_readiness: todo!(),
    payload: todo!(),
};
```

Raw JSON cannot satisfy page-header native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

let raw = serde_json::Value::Null;
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::PhysicalPageHeader,
)
.with_page_header_witness(raw);
```

Raw JSON cannot satisfy aspect-boundary native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

let raw = serde_json::Value::Null;
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::AspectBoundaryFact,
)
.with_aspect_boundary_fact(raw);
```

Raw JSON cannot satisfy aspect-patch native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

let raw = serde_json::Value::Null;
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::AspectPatchBoundaryFact,
)
.with_aspect_patch_boundary_fact(raw);
```

String text cannot satisfy page-header native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

let text = String::from("store.physical.page.header");
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::PhysicalPageHeader,
)
.with_page_header_witness(text);
```

String text cannot satisfy aspect-boundary native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

let text = String::from("store.aspect.boundary.fact");
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::AspectBoundaryFact,
)
.with_aspect_boundary_fact(text);
```

String text cannot satisfy aspect-patch native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

let text = String::from("store.aspect.patch.boundary.fact");
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::AspectPatchBoundaryFact,
)
.with_aspect_patch_boundary_fact(text);
```

Terminal projection text cannot satisfy page-header native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily, StoreTerminalProjectionText,
};

let text: StoreTerminalProjectionText = todo!();
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::PhysicalPageHeader,
)
.with_page_header_witness(text);
```

Terminal projection text cannot satisfy aspect-boundary native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily, StoreTerminalProjectionText,
};

let text: StoreTerminalProjectionText = todo!();
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::AspectBoundaryFact,
)
.with_aspect_boundary_fact(text);
```

Terminal projection text cannot satisfy aspect-patch native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily, StoreTerminalProjectionText,
};

let text: StoreTerminalProjectionText = todo!();
let _ = StoreCanonicalBasisConstruction::for_family(
    StoreCanonicalBasisFamily::AspectPatchBoundaryFact,
)
.with_aspect_patch_boundary_fact(text);
```

Generic Serialize inputs cannot satisfy page-header native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

fn try_generic_serialize<T: serde::Serialize>(value: T) {
    let _ = StoreCanonicalBasisConstruction::for_family(
        StoreCanonicalBasisFamily::PhysicalPageHeader,
    )
    .with_page_header_witness(value);
}
```

Generic Serialize inputs cannot satisfy aspect-boundary native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

fn try_generic_serialize<T: serde::Serialize>(value: T) {
    let _ = StoreCanonicalBasisConstruction::for_family(
        StoreCanonicalBasisFamily::AspectBoundaryFact,
    )
    .with_aspect_boundary_fact(value);
}
```

Generic Serialize inputs cannot satisfy aspect-patch native Store canonical basis construction:

```compile_fail
use worth_store_aspect_native::{
    StoreCanonicalBasisConstruction, StoreCanonicalBasisFamily,
};

fn try_generic_serialize<T: serde::Serialize>(value: T) {
    let _ = StoreCanonicalBasisConstruction::for_family(
        StoreCanonicalBasisFamily::AspectPatchBoundaryFact,
    )
    .with_aspect_patch_boundary_fact(value);
}
```

Digest strings cannot construct Store aspect identity authority:

```compile_fail
use worth_store_aspect_native::StoreAspectIdentity;

let digest_text = String::from("sha256:aspect-identity");
let _identity = StoreAspectIdentity::from_digest_text(digest_text);
```

Digest strings cannot construct Store recovery source authority:

```compile_fail
use worth_store_recovery_physics::PhysicalRecoverySource;

let digest_text = String::from("sha256:recovery-source");
let _source = PhysicalRecoverySource::from_digest_text(digest_text);
```

Digest strings cannot construct Store checkpoint authority:

```compile_fail
use worth_store_recovery_physics::CheckpointValidityDecision;

let digest_text = String::from("sha256:checkpoint");
let _checkpoint = CheckpointValidityDecision::from_digest_text(digest_text);
```

Digest strings cannot construct Store page authority:

```compile_fail
use worth_store_physical_format::PhysicalPageId;

let digest_text = String::from("sha256:page");
let _page = PhysicalPageId::from_digest_text(digest_text);
```

Certification cannot skip the readiness-owned S.7 capsule handoff:

```compile_fail
use worth_store_blob_chunks::BlobCapsuleReadinessWitness;
use worth_store_certification::certify_s7_capsule_readiness;

let readiness: BlobCapsuleReadinessWitness = todo!();
let _report = certify_s7_capsule_readiness(&readiness);
```
