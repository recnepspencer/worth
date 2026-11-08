from pathlib import Path
import json,base64,hashlib,sys
backup=Path('target/lane-evidence/area-funding-instrumentation-originals.json')
if sys.argv[1]=='restore':
 saved=json.loads(backup.read_text())
 for name,data in saved['originals'].items():
  Path(name).write_bytes(base64.b64decode(data))
  assert hashlib.sha256(Path(name).read_bytes()).hexdigest()==saved['sha256'][name]
 for name in saved['created']:
  p=Path(name).resolve()
  assert p.is_relative_to(Path.cwd().resolve()) and p.suffix=='.rs'
  p.unlink()
 print('Restored exact original source bytes; removed the diagnostic-only recipe.')
 raise SystemExit(0)
assert not backup.exists(), 'do not overwrite a diagnostic backup'
changes={};original={}
def edit(name,transform):
 p=Path(name);original[name]=p.read_bytes();changes[name]=transform(p.read_text())
def replace(s,a,b):
 assert s.count(a)==1,(a,s.count(a));return s.replace(a,b)
helper="""
// WORTH-UI-TEMPORARY-INSTRUMENTATION: read-only charged-work observation.
fn funding_work(request: &ExecutionRequest<'_, '_>) -> u64 {
 let mut value = None;
 request.run_native_with_checkpoint(|context| value=context.remaining_work()).unwrap();
 value.expect("finite diagnostic meter")
}
fn funding_phase(name:&str, prior:&mut u64, request:&ExecutionRequest<'_, '_>) {
 let next=funding_work(request);
 eprintln!("WORTH-UI-TEMPORARY-INSTRUMENTATION PHASE {name} units={}",*prior-next);
 *prior=next;
}
"""
def native(s):
 s=replace(s,'    let mut frozen =','    let mut funding=funding_work(execution);\n    let mut frozen =')
 points=[('    let commits =','native/freeze_indexes'),('    let dictionary =','native/uncovered_original_readmission'),('    let roots =','native/dictionary'),('    let main =','native/held_roots'),('    let schemas =','native/branch_state'),('    let initial =','native/schemas'),('    let original_image =','native/initial_inputs'),('    let lineage =','native/normalized_originals'),('    let canonical =','native/history_metadata_indexes'),('    let region_output =','native/canonical_inventory'),('    let main_schema =','native/current_regions'),('    let (bytes, sections) =','native/manifest'),('    let publication =','native/checkpoint_serialization'),('    let source_artifact =','native/selection_custody'),('    let verified =','native/complete_source_serialization'),('    let admission_image =','native/verified_custody'),('    let (checkpoint, source_artifact) =','native/admission_projection')]
 for token,name in points:s=replace(s,token,'    funding_phase("'+name+'", &mut funding, execution);\n'+token)
 return s+helper
edit('crates/worth-relational/src/durability/authority/checkpoint_complete/prepare.rs',native)
def store(s):
 s=replace(s,'    let world = world\n','    let mut funding=funding_work(execution);\n    let world = world\n')
 s=replace(s,'    capture_after_world(serving, execution, world, native)','    funding_phase("store/world_image", &mut funding, execution);\n    capture_after_world(serving, execution, world, native)')
 s=replace(s,'    let (home, through, world_digest) = world','    let mut funding=funding_work(execution);\n    let (home, through, world_digest) = world')
 for token,name in [('    let native = native(home, through, world_digest)','store/world_coordinates'),('    let image = native.admission_image();','store/native_and_image_seal'),('    let native = native\n','store/world_companion'),('    // Native owns','store/package_hash_copy'),('    frame(native, world, previous.as_ref(), execution)','store/previous_selection')]:s=replace(s,token,'    funding_phase("'+name+'", &mut funding, execution);\n'+token)
 s=replace(s,'    let generation = previous.map_or','    let mut funding=funding_work(execution);\n    let generation = previous.map_or')
 s=replace(s,'    let identity = publication_identity','    funding_phase("store/manifest_copy_members", &mut funding, execution);\n    let identity = publication_identity')
 s=replace(s,'    Ok(PreparedHomeCheckpoint {','    funding_phase("store/manifest_hash", &mut funding, execution);\n    Ok(PreparedHomeCheckpoint {')
 return s+helper
edit('workspaces/worth-store/crates/worth-store-application/src/checkpoint/preparation.rs',store)
mod='workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/host/application_home/reopening.rs'
edit(mod,lambda s:replace(s,'mod checkpoint_restart;','mod checkpoint_restart;\nmod funding_probe;'))
probe='workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/host/application_home/reopening/funding_probe.rs'
assert not Path(probe).exists()
changes[probe]=Path('target/lane-evidence/area-funding-probe-recipe.txt').read_text()
if sys.argv[1]=='check':
 print('Diagnostic anchors validated; no source files changed:',len(changes))
 raise SystemExit(0)
saved={'originals':{n:base64.b64encode(v).decode() for n,v in original.items()},'sha256':{n:hashlib.sha256(v).hexdigest() for n,v in original.items()},'created':[probe]}
backup.write_text(json.dumps(saved,indent=2)+'\n')
for n,s in changes.items():Path(n).write_text(s)
print('Applied diagnostic-only owner phase meters to2 granted owners and the Host court recipe.')
