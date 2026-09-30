Every use of an output role names its declaration marker, and a use that
disagrees with the output contract fails to compile. The passing twins below
use each declared role and family as the contract declares it; each failing
example changes one thing.

Fixed roles, bound by a handler and read from a commit receipt. A receipt
stores its outputs erased; `outputs_of::<Contract>()` checks the commit's
contract once and returns a view on which every read is checked against that
contract at compile time:

```
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{
#     CandidateWriter, WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectEntity,
# };
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.preserve_output::<PlanarAnchorOutput<S>>(body);
}

fn bind_final(
    writer: &mut CandidateWriter<'_, S, FinalPlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_output::<FinalAnchorOutput<S>>(body);
    let _ = writer.create_output::<FinalClosingOutput<S>>(body);
}

fn read(receipt: &WorthQueryApplicationCommitReceipt) {
    if let Ok(outputs) = receipt.outputs_of::<PlanarOutputs>() {
        let _ = outputs.entity::<PlanarAnchorOutput<S>>();
    }
    if let Ok(outputs) = receipt.outputs_of::<FinalPlanarOutputs>() {
        let _ = outputs.entity::<FinalClosingOutput<S>>();
    }
}
# let _ = bind as fn(_, _);
# let _ = bind_final as fn(_, _);
# let _ = read as fn(_);
```

A role its contract does not declare:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     WorthQueryApplicationOutputRole, WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct Stray;
impl WorthQueryApplicationOutputRole for Stray {
    type Schema = S;
    type Contract = PlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "stray";
}

fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.preserve_output::<Stray>(body);
}
# let _ = bind as fn(_, _);
```

A declared role read as another entity:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     WorthQueryApplicationOutputRole, WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::WorthQueryApplicationTypedOutputCorrespondence;
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct AnchorAsPrincipal;
impl WorthQueryApplicationOutputRole for AnchorAsPrincipal {
    type Schema = S;
    type Contract = PlanarOutputs;
    type Entity = Principal;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "anchor";
}

fn read(outputs: WorthQueryApplicationTypedOutputCorrespondence<'_, PlanarOutputs>) {
    let _ = outputs.entity::<AnchorAsPrincipal>();
}
# let _ = read as fn(_);
```

An at-most-one role used as exactly-one:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct ClosingAsRequired;
impl WorthQueryApplicationOutputRole for ClosingAsRequired {
    type Schema = S;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "closing";
}

fn bind(
    writer: &mut CandidateWriter<'_, S, FinalPlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_output::<ClosingAsRequired>(body);
}
# let _ = bind as fn(_, _);
```

A role declared with another posture:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct CreatedAnchor;
impl WorthQueryApplicationOutputRole for CreatedAnchor {
    type Schema = S;
    type Contract = PlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "anchor";
}

fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_output::<CreatedAnchor>(body);
}
# let _ = bind as fn(_, _);
```

A declared role bound with a posture other than its own:

```compile_fail,E0271
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_output::<PlanarAnchorOutput<S>>(body);
}
# let _ = bind as fn(_, _);
```

A role of another operation's contract:

```compile_fail,E0271
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.preserve_output::<VertexReplacementAnchorOutput<S>>(body);
}
# let _ = bind as fn(_, _);
```

A role of another operation's contract, read on this contract's view:

```compile_fail,E0271
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::WorthQueryApplicationTypedOutputCorrespondence;
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn read(outputs: WorthQueryApplicationTypedOutputCorrespondence<'_, PlanarOutputs>) {
    let _ = outputs.entity::<FinalAnchorOutput<S>>();
}
# let _ = read as fn(_);
```

Families, bound by suffix and read back:

```
# use worth_query_host::facade::declaration::application_operation::WorthQueryCreateOutput;
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{
#     CandidateWriter, WorthQueryApplicationEffectEntity,
#     WorthQueryApplicationTypedOutputCorrespondence,
# };
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_member::<PlanarCreatedOutputs<S>>("vertex", body);
}

fn read(outputs: WorthQueryApplicationTypedOutputCorrespondence<'_, PlanarOutputs>) {
    let _ = outputs.member::<PlanarCreatedOutputs<S>, WorthQueryCreateOutput>("vertex");
    let _ = outputs.family_entries::<PlanarCreatedOutputs<S>>();
}
# let _ = bind as fn(_, _);
# let _ = read as fn(_);
```

A family its contract does not declare:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     ApplicationMutationOutputPostureSet, WorthQueryApplicationOutputRoleFamily,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct Stray;
impl WorthQueryApplicationOutputRoleFamily for Stray {
    type Schema = S;
    type Contract = PlanarOutputs;
    type Entity = Body;
    const PREFIX: &'static str = "stray.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 0;
}

fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_member::<Stray>("vertex", body);
}
# let _ = bind as fn(_, _);
```

A declared family read as another entity:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     ApplicationMutationOutputPostureSet, WorthQueryApplicationOutputRoleFamily,
#     WorthQueryCreateOutput,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::WorthQueryApplicationTypedOutputCorrespondence;
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct CreatedPrincipals;
impl WorthQueryApplicationOutputRoleFamily for CreatedPrincipals {
    type Schema = S;
    type Contract = PlanarOutputs;
    type Entity = Principal;
    const PREFIX: &'static str = "created.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 0;
}

fn read(outputs: WorthQueryApplicationTypedOutputCorrespondence<'_, PlanarOutputs>) {
    let _ = outputs.member::<CreatedPrincipals, WorthQueryCreateOutput>("vertex");
}
# let _ = read as fn(_);
```

A family declared with other postures:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     ApplicationMutationOutputPostureSet, WorthQueryApplicationOutputRoleFamily,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct AnyPostureCreated;
impl WorthQueryApplicationOutputRoleFamily for AnyPostureCreated {
    type Schema = S;
    type Contract = PlanarOutputs;
    type Entity = Body;
    const PREFIX: &'static str = "created.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 0;
}

fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_member::<AnyPostureCreated>("vertex", body);
}
# let _ = bind as fn(_, _);
```

A family declared with another minimum member count:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_operation::{
#     ApplicationMutationOutputPostureSet, WorthQueryApplicationOutputRoleFamily,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
struct AtLeastOneCreated;
impl WorthQueryApplicationOutputRoleFamily for AtLeastOneCreated {
    type Schema = S;
    type Contract = PlanarOutputs;
    type Entity = Body;
    const PREFIX: &'static str = "created.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 1;
}

fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_member::<AtLeastOneCreated>("vertex", body);
}
# let _ = bind as fn(_, _);
```

A member bound with a posture outside its family's set:

```compile_fail,E0080
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.preserve_member::<PlanarCreatedOutputs<S>>("vertex", body);
}
# let _ = bind as fn(_, _);
```

A family of another operation's contract:

```compile_fail,E0271
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::{CandidateWriter, WorthQueryApplicationEffectEntity};
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn bind(
    writer: &mut CandidateWriter<'_, S, PlanarMutationBinding<S>>,
    body: &WorthQueryApplicationEffectEntity<S, Body>,
) {
    let _ = writer.create_member::<FinalCreatedOutputs<S>>("vertex", body);
}
# let _ = bind as fn(_, _);
```

A member of another operation's family, read on this contract's view:

```compile_fail,E0271
# use worth_query_host::facade::declaration::application_operation::WorthQueryCreateOutput;
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::WorthQueryApplicationTypedOutputCorrespondence;
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn read(outputs: WorthQueryApplicationTypedOutputCorrespondence<'_, PlanarOutputs>) {
    let _ = outputs.member::<FinalCreatedOutputs<S>, WorthQueryCreateOutput>("vertex");
}
# let _ = read as fn(_);
```

The entries of another operation's family, read on this contract's view:

```compile_fail,E0271
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_host::facade::primary_graph::WorthQueryApplicationTypedOutputCorrespondence;
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
fn read(outputs: WorthQueryApplicationTypedOutputCorrespondence<'_, PlanarOutputs>) {
    let _ = outputs.family_entries::<FinalCreatedOutputs<S>>();
}
# let _ = read as fn(_);
```

A producer names the exactly-one role of its operation's contract that
binds its family's entity, and installing it checks that role against the
contract:

```
# use worth_query_host::facade::application_contribution::{
#     WorthQueryApplicationContributionContracts, WorthQueryApplicationProducerBinding,
#     WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
#     WorthQueryProducerDemandResources, WorthQueryProducerInvariantRequirement,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
# struct Provider;
# impl WorthQueryApplicationProducerProvider<S, Producer> for Provider {
#     const SEMANTIC_IDENTITY: &'static str = "doc.provider";
#     fn operation_input(&self, _: &PlanarReadResult) -> FinalPlanarMutation {
#         unimplemented!()
#     }
#     fn idempotency_key(&self, _: &PlanarReadResult, _: &[u8; 32]) -> u64 {
#         unimplemented!()
#     }
#     fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
#         unimplemented!()
#     }
# }
struct Producer;
impl WorthQueryApplicationProducerBinding<S> for Producer {
    type Operation = FinalPlanarMutationBinding<S>;
    type OutputFamily = PlanarFinalOutputFamily;
    type Provider = Provider;
    type OutputRole = FinalAnchorOutput<S>;
#   const IDENTITY: &'static str = "doc.producer";
#   const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[];
#   const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
#   const RESOURCE_POLICY: &'static str = "bounded-synchronous";
#   const REUSE_POLICY: &'static str = "exact-source";
}

fn install(contracts: &mut WorthQueryApplicationContributionContracts<S>) {
    let _ = contracts.producer::<Producer>();
}
# let _ = install as fn(_);
```

A producer whose role is at-most-one:

```compile_fail,E0271
# use worth_query_host::facade::application_contribution::{
#     WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
#     WorthQueryProducerApplicability, WorthQueryProducerDemandResources,
#     WorthQueryProducerInvariantRequirement,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
# struct Provider;
# impl WorthQueryApplicationProducerProvider<S, Producer> for Provider {
#     const SEMANTIC_IDENTITY: &'static str = "doc.provider";
#     fn operation_input(&self, _: &PlanarReadResult) -> FinalPlanarMutation {
#         unimplemented!()
#     }
#     fn idempotency_key(&self, _: &PlanarReadResult, _: &[u8; 32]) -> u64 {
#         unimplemented!()
#     }
#     fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
#         unimplemented!()
#     }
# }
struct Producer;
impl WorthQueryApplicationProducerBinding<S> for Producer {
    type Operation = FinalPlanarMutationBinding<S>;
    type OutputFamily = PlanarFinalOutputFamily;
    type Provider = Provider;
    type OutputRole = FinalClosingOutput<S>;
#   const IDENTITY: &'static str = "doc.producer";
#   const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[];
#   const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
#   const RESOURCE_POLICY: &'static str = "bounded-synchronous";
#   const REUSE_POLICY: &'static str = "exact-source";
}
```

A producer whose role binds another entity than its family outputs:

```compile_fail,E0271
# use worth_query_host::facade::application_contribution::{
#     WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
#     WorthQueryProducerApplicability, WorthQueryProducerDemandResources,
#     WorthQueryProducerInvariantRequirement, WorthQueryProducerOutputFamily,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
# struct Provider;
# impl WorthQueryApplicationProducerProvider<S, Producer> for Provider {
#     const SEMANTIC_IDENTITY: &'static str = "doc.provider";
#     fn operation_input(&self, _: &PlanarReadResult) -> FinalPlanarMutation {
#         unimplemented!()
#     }
#     fn idempotency_key(&self, _: &PlanarReadResult, _: &[u8; 32]) -> u64 {
#         unimplemented!()
#     }
#     fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
#         unimplemented!()
#     }
# }
struct PrincipalOutputs;
impl WorthQueryProducerOutputFamily<S> for PrincipalOutputs {
    type Source = PlanarReadBinding<S>;
    type Entity = Principal;
#   const IDENTITY: &'static str = "doc.principal-outputs";
#   const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[];
#   fn profile_kind(_: &PlanarReadResult) -> &'static str {
#       "doc"
#   }
}

struct Producer;
impl WorthQueryApplicationProducerBinding<S> for Producer {
    type Operation = FinalPlanarMutationBinding<S>;
    type OutputFamily = PrincipalOutputs;
    type Provider = Provider;
    type OutputRole = FinalAnchorOutput<S>;
#   const IDENTITY: &'static str = "doc.producer";
#   const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[];
#   const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
#   const RESOURCE_POLICY: &'static str = "bounded-synchronous";
#   const REUSE_POLICY: &'static str = "exact-source";
}
```

A producer whose role belongs to another operation's contract:

```compile_fail,E0271
# use worth_query_host::facade::application_contribution::{
#     WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
#     WorthQueryProducerApplicability, WorthQueryProducerDemandResources,
#     WorthQueryProducerInvariantRequirement,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
# struct Provider;
# impl WorthQueryApplicationProducerProvider<S, Producer> for Provider {
#     const SEMANTIC_IDENTITY: &'static str = "doc.provider";
#     fn operation_input(&self, _: &PlanarReadResult) -> FinalPlanarMutation {
#         unimplemented!()
#     }
#     fn idempotency_key(&self, _: &PlanarReadResult, _: &[u8; 32]) -> u64 {
#         unimplemented!()
#     }
#     fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
#         unimplemented!()
#     }
# }
struct Producer;
impl WorthQueryApplicationProducerBinding<S> for Producer {
    type Operation = FinalPlanarMutationBinding<S>;
    type OutputFamily = PlanarFinalOutputFamily;
    type Provider = Provider;
    type OutputRole = PlanarAnchorOutput<S>;
#   const IDENTITY: &'static str = "doc.producer";
#   const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[];
#   const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
#   const RESOURCE_POLICY: &'static str = "bounded-synchronous";
#   const REUSE_POLICY: &'static str = "exact-source";
}
```

A producer whose role its contract does not declare:

```compile_fail,E0080
# use worth_query_host::facade::application_contribution::{
#     WorthQueryApplicationContributionContracts, WorthQueryApplicationProducerBinding,
#     WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
#     WorthQueryProducerDemandResources, WorthQueryProducerInvariantRequirement,
# };
# use worth_query_host::facade::declaration::application_operation::{
#     WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
# };
# use worth_query_host::facade::declaration::application_schema::{
#     ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
# };
# use worth_query_topology_entry::*;
# struct S;
# impl ApplicationSchema for S {
#     const OWNER: &'static str = "doc";
#     const NAME: &'static str = "S";
#     const MAJOR: u32 = 1;
#     const MINOR: u32 = 0;
#     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
#         unimplemented!()
#     }
# }
# impl TopologySchemaBinding for S {}
# struct Provider;
# impl WorthQueryApplicationProducerProvider<S, Producer> for Provider {
#     const SEMANTIC_IDENTITY: &'static str = "doc.provider";
#     fn operation_input(&self, _: &PlanarReadResult) -> FinalPlanarMutation {
#         unimplemented!()
#     }
#     fn idempotency_key(&self, _: &PlanarReadResult, _: &[u8; 32]) -> u64 {
#         unimplemented!()
#     }
#     fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
#         unimplemented!()
#     }
# }
struct Undeclared;
impl WorthQueryApplicationOutputRole for Undeclared {
    type Schema = S;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "undeclared";
}

struct Producer;
impl WorthQueryApplicationProducerBinding<S> for Producer {
    type Operation = FinalPlanarMutationBinding<S>;
    type OutputFamily = PlanarFinalOutputFamily;
    type Provider = Provider;
    type OutputRole = Undeclared;
#   const IDENTITY: &'static str = "doc.producer";
#   const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[];
#   const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
#   const RESOURCE_POLICY: &'static str = "bounded-synchronous";
#   const REUSE_POLICY: &'static str = "exact-source";
}

fn install(contracts: &mut WorthQueryApplicationContributionContracts<S>) {
    let _ = contracts.producer::<Producer>();
}
# let _ = install as fn(_);
```
