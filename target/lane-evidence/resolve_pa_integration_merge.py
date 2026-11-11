"""Resolve the authorized integration merge by claim and provider protocol owner."""
from pathlib import Path
import subprocess,re,json,sys
sys.path.insert(0,'scripts/quality')
from rust_function_scrutiny import sanitize_rust,signature_terminator,matching_delimiter
E=Path('target/lane-evidence');R=Path('crates/worth-relational/src')
P=Path('crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication')
assert subprocess.check_output(['git','rev-parse','MERGE_HEAD'],text=True).strip()=='22e638d638e724fc2ca3947a583f43c45b56990f'
archive=E/'pa-merge-original-conflicts';archive.mkdir(exist_ok=True)
names=subprocess.check_output(['git','diff','--name-only','--diff-filter=U'],text=True).splitlines()
for i,name in enumerate(names):
 if Path(name).exists():(archive/(str(i)+'.txt')).write_text(Path(name).read_text())
def stage(which,p):return subprocess.check_output(['git','show',':'+str(which)+':'+p.as_posix()]).decode()
def grab(s,name,occ=0):
 c=sanitize_rust(s);m=list(re.finditer(r'\bfn\s+'+re.escape(name)+r'\s*\(',c))[occ]
 a=s.rfind('\n',0,m.start())+1;b=matching_delimiter(c,signature_terminator(c,m.end()-1),'{','}')+1
 return s[a:b]
pattern=r'<<<<<<<[^\n]*\n(.*?)=======\n(.*?)>>>>>>>[^\n]*\n'
def resolve(p,choose):
 s=p.read_text();blocks=list(re.finditer(pattern,s,re.S));assert blocks,p
 for i,m in reversed(list(enumerate(blocks))):s=s[:m.start()]+choose(i,m[1],m[2])+s[m.end():]
 assert not re.search(r'^<<<<<<<|^=======|^>>>>>>>',s,re.M);p.write_text(s)
resolved=[]
# Retired campaigns remain retired. Keep all automatic integration API updates.
for f in ['durability/authority/runtime_rebuild/checkpoint_restore_tests.rs','durability/log/native_checkpoint_codec/kind_authentication_tests.rs','history/commit/artifact_tests.rs','history/commit/envelope_residency_tests.rs','runtime/state/subsystems/services/symbol_table/reservations/tests.rs','tests/durability/contracts/managed_field_text.rs']:
 p=R/f;resolve(p,lambda i,a,b:a);resolved.append(p.as_posix())
p=R/'durability/authority/runtime_rebuild/checkpoint_restore_tests.rs';s=p.read_text();s=s.replace('use checkpoint_substrate_identity::{shares_first_generation, shares_partition_generation};\n','');p.write_text(s)
# Public ports take integration's new semantic assertions without private factories.
p=P/'checkpoint_custom_versions.rs';resolve(p,lambda i,a,b:b.replace('crate::test_execution_policy::serial_request()', 'worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request()'))
s=p.read_text();s=s.replace('let (execution_meaning, custom) = semantic','let (execution_index, execution_meaning, custom) = semantic')
s=s.replace('''        .iter()
        .find_map(|meaning| {''','''        .iter()
        .enumerate()
        .find_map(|(execution_index, meaning)| {''',1)
s=s.replace('.map(|custom| (meaning, custom))','.map(|custom| (execution_index, meaning, custom))');p.write_text(s);resolved.append(p.as_posix())
p=P/'checkpoint_generation_identity.rs'
resolve(p,lambda i,a,b:b[:b.index('    let committed =')]+a);resolved.append(p.as_posix())
p=P/'checkpoint_index_lagging.rs';resolve(p,lambda i,a,b:a)
s=p.read_text().replace('fn lagging_inherited_and_own_stream_scoped_generations_survive_checkpoint_reopen()', 'fn retained_inherited_and_own_stream_generations_survive_native_maintenance_and_checkpoint_reopen()');p.write_text(s);resolved.append(p.as_posix())
# Both source-read funding and integration's retirement callbacks are mandatory.
for f in ['storage/residency/head_probe_tests.rs','storage/residency/request_stop_tests.rs']:
 p=R/f;resolve(p,lambda i,a,b:a+'    }\n\n'+b);resolved.append(p.as_posix())
# Keep integration's actual controlled-retirement owner and new tests, then restore funded reads.
p=R/'storage/residency/region_fixture.rs';ours=stage(2,p);theirs=stage(3,p)
owned=grab(ours,'read_member_owned');member=grab(ours,'member')
funded_body=owned[owned.index('        let bytes ='):owned.rfind('\n    }')]
funded='''    fn funded_member(&self, request: &worth_execution::ExecutionRequest<'_, '_>, locator: RelationalDurableMemberLocator, limit: u64) -> Result<worth_execution::ExecutionImmutableBytes, crate::durability::medium::RelationalDurableOwnedReadRefusal> {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
'''+funded_body+'\n    }'
owned_wrapper='''    fn read_member_owned(&self, request: &worth_execution::ExecutionRequest<'_, '_>, original: &(dyn std::any::Any + Send + Sync), locator: RelationalDurableMemberLocator, limit: u64) -> Result<worth_execution::ExecutionImmutableBytes, crate::durability::medium::RelationalDurableOwnedReadRefusal> {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        request.check_live().map_err(|cause| Refusal::Request(cause.into()))?;
        if !original.is::<()>() { return Err(Refusal::ForeignProtection); }
        self.funded_member(request, locator, limit)
    }
'''
anchor='impl RelationalDurableColdSource for CountedSource {\n';assert anchor in theirs;theirs=theirs.replace(anchor,anchor+owned_wrapper,1)
theirs+='\nimpl CountedSource {\n'+member+'\n'+funded+'\n}\n'
controlled_read='''    fn read_member_owned(&self, request: &worth_execution::ExecutionRequest<'_, '_>, original: &(dyn std::any::Any + Send + Sync), locator: RelationalDurableMemberLocator, limit: u64) -> Result<worth_execution::ExecutionImmutableBytes, crate::durability::medium::RelationalDurableOwnedReadRefusal> {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        request.check_live().map_err(|cause| Refusal::Request(cause.into()))?;
        let owner = original.downcast_ref::<ControlledCustodyOwner>().ok_or(Refusal::ForeignProtection)?;
        if !owner.registered || !Arc::ptr_eq(&owner.active, &self.active) || owner.source_issuer.as_ptr() != Arc::as_ptr(&self.source) { return Err(Refusal::ForeignProtection); }
        // The provider validates its actual original registration, then reads its immutable archive.
        // No manufactured child token or alternate issuer participates.
        self.source.funded_member(request, locator, limit)
    }
'''
anchor='impl RelationalDurableColdSource for CustodySource {\n';assert anchor in theirs;theirs=theirs.replace(anchor,anchor+controlled_read,1)
# Plain payload/Drop bookkeeping has no runtime/request construction root of its own.
a=theirs.index('// The real registration is disarmed');b=theirs.index('impl RelationalDurableColdSource for CustodySource',a)
owners=theirs[a:b];theirs=theirs[:a]+theirs[b:]
owners=owners.replace('struct ControlledCustodyOwner {','pub(super) struct ControlledCustodyOwner {').replace('struct CustodyOwner {','pub(super) struct CustodyOwner {')
owners=re.sub(r'^(    )(source_issuer|active|registered|child|_source):',r'\1pub(super) \2:',owners,flags=re.M)
leaf=p.parent/'region_fixture/placement_registration.rs';leaf.parent.mkdir(exist_ok=True)
leaf.write_text('//! Original placement registration payloads and abandonment disposal.\nuse super::CountedSource;\nuse crate::durability::medium::RelationalDurableMemberCustody;\nuse std::sync::{Arc, atomic::{AtomicUsize, Ordering}};\n\n'+owners)
theirs=theirs.replace('use super::super::{capture_region_members, RelationalColdRegion, RelationalRegionResidency};','use super::super::{capture_region_members, RelationalColdRegion, RelationalRegionResidency};\nmod placement_registration;\nuse placement_registration::{ControlledCustodyOwner, CustodyOwner};');p.write_text(theirs)
resolved.extend([p.as_posix(),leaf.as_posix()])
p=R/'storage/residency/member_custody_tests.rs';p.write_text(stage(3,p));resolved.append(p.as_posix())
# Preserve the intended-cause malformed-column guard and the new explicit oracle lane.
p=R/'visibility/materialization/read_records/projection/field_revisions/restore_tests.rs'
resolve(p,lambda i,a,b:'    assert_eq!(image.field_revisions.len() + 1, image.generations.len());\n'+b);resolved.append(p.as_posix())
# Existing source-root/caller registries merge by exact entries, never wildcard growth.
p=Path('tools/boundary-check/config/certification-construction-callers.txt')
resolve(p,lambda i,a,b:'\n'.join(dict.fromkeys((a+b).splitlines()))+'\n');resolved.append(p.as_posix())
deleted=[P/'artifact_affinity.rs',R/'storage/residency/cross_region_forgery_tests.rs',R/'tests/durability/contracts/derived_index_checkpoint.rs',R/'tests/support/durability/canonical_commit_frame.rs',R/'durability/authority/runtime_rebuild/checkpoint_substrate_identity.rs',R/'durability/authority/runtime_rebuild/checkpoint_restore/reuse_tests.rs']
workspace=Path.cwd().resolve()
for p in deleted:
 assert p.resolve().is_relative_to(workspace),p
 if p.exists():p.unlink()
subprocess.run(['git','add','-A','--',*[p.as_posix() for p in deleted]],check=True)
p=Path('tools/boundary-check/config/src-court-baseline.json');resolve(p,lambda i,a,b:a+b)
rows=json.loads(p.read_text());rows=[r for r in rows if Path(r['source']).exists()]
unique={}
for r in rows:
 key=(r['source'],r['module'])
 if key in unique:assert unique[key]==r,key
 unique[key]=r
p.write_text('[\n'+',\n'.join(json.dumps(r,separators=(',',':')) for r in unique.values())+'\n]\n');resolved.append(p.as_posix())
ledger={'integration':'22e638d638e724fc2ca3947a583f43c45b56990f','resolved':resolved,'kept_retired':[p.as_posix() for p in deleted],'retirement_refresh':{'C2':['durability/authority/runtime_rebuild/checkpoint_restore_tests.rs','durability/authority/runtime_rebuild/checkpoint_restore/reuse_tests.rs','history/commit/artifact_tests.rs','history/commit/envelope_residency_tests.rs','runtime/state/subsystems/services/symbol_table/reservations/tests.rs'],'C4':['durability/authority/runtime_rebuild/checkpoint_restore/reuse_tests.rs','durability/log/native_checkpoint_codec/kind_authentication_tests.rs','history/commit/envelope_residency_tests.rs','storage/residency/cross_region_forgery_tests.rs','tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs','tests/durability/contracts/semantic_result/custom_versions.rs','tests/support/durability/canonical_commit_frame.rs']}}
# Refresh immutable private campaign closures to the exact merged integration protocol.
p=Path('plans/database-foundation/milestone-3.md');s=p.read_text()
for band,heading in [('C2','### C2 - Cost and bounds: backing and work budgets\n'),('C4','### C4 - Injected faults\n')]:
 assert heading in s
 entries='\n'.join('- `22e638d638e724fc2ca3947a583f43c45b56990f:crates/worth-relational/src/'+f+'`' for f in ledger['retirement_refresh'][band])
 text='\nIntegration protocol restoration refresh (same per-claim disposition; no retired campaign is remounted):\n'+entries+'\nRestore the complete same-SHA `crates/worth-relational/src` tree, including original parent mounts, fixture/control/support/producer and reconstruction-lane dependencies. Preserve every original assertion/input/contrast and genuinely owned source boundary when migrating the campaign; never revive runtime_attachment discovery. These references retain integration revisions to the already moved private cost/fault families. Ordinary source functionality remains in the public counterparts.\n\n'
 s=s.replace(heading,heading+text,1)
p.write_text(s)
(E/'pa-integration-resolution-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n')
subprocess.run(['git','add','--',*resolved,p.as_posix()],check=True)
remaining=subprocess.check_output(['git','diff','--name-only','--diff-filter=U'],text=True).strip();assert not remaining,remaining
print('Resolved20conflicts; retained public ports and funded reads plus integration retirement; refreshed C2/C4 protocol closures')
