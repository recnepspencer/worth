"""Refresh the seven artifact identities from owned Cargo/nextest results."""
import gc,json,struct,subprocess,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';base=e/'layout-r3';sys.path.insert(0,str(e))
from lifecycle_compiled_frames import Frames
programs=json.loads((base/'ordinary/actual-entry-artifacts.json').read_text())
manifest={name:program['exe'] for name,program in programs.items()}
for owner,label,target in [('root','root-court','worth_court_runtime_bridge'),('query','query-court','worth_query_court_query_decl_host'),('bank','bank','worth_query_bank_court_external_rail_server')]:
 owner_base=e/'layout-r1' if owner=='root' else base
 binaries=json.loads((owner_base/owner/'build/binaries.json').read_text())['rust-binaries']
 matches=[b['binary-path'] for b in binaries.values() if b['binary-name']==target]
 assert len(matches)==1,(owner,matches)
 manifest[label]=matches[0]
assert len(manifest)==7
(e/'layout-after-frame-inputs.json').write_text(json.dumps(manifest,indent=2))
reserves={}
for name in ('invariant_reads','required_feature','shared_execution','worth-query-consumer-root'):
 pe=Frames(manifest[name]);reserves[name]=struct.unpack_from('<Q',pe.data,pe.u32(0x3c)+24+72)[0];del pe;gc.collect()
assert set(reserves.values())=={1048576},reserves
(e/'layout-main-stack-reserves.json').write_text(json.dumps(reserves,indent=2))
# The four ordinary artifacts were already scanned/identified at the same production inputs.
# Reuse their exact PE/PDB hash-bound records; scan only the three new court artifacts.
import hashlib
prior=json.loads((e/'layout-r3-four-identified.json').read_text())
for binary in prior['binaries']:
    assert manifest[binary['binary']]==binary['exe']
    identity=binary['artifact_identity']
    assert hashlib.file_digest(open(binary['exe'],'rb'),'sha256').hexdigest()==identity['exe_sha256']
    assert hashlib.file_digest(open(identity['pdb_path'],'rb'),'sha256').hexdigest()==identity['pdb_sha256']
remaining={name:path for name,path in manifest.items() if name not in {b['binary'] for b in prior['binaries']}}
(e/'layout-after-court-inputs.json').write_text(json.dumps(remaining,indent=2))
commands=[
 ['whole_binary_frames.py','layout-after-court-inputs.json','layout-after-court-frames.json'],
 ['pdb_public_frames.py','layout-after-court-frames.json','layout-after-court-publics.json'],
 ['coff_frame_attribution.py','layout-after-court-publics.json','layout-after-court-identified.json']]
for command in commands:
 code=subprocess.call([sys.executable,*[str(e/item) for item in command]])
 if code:raise SystemExit(code)
courts=json.loads((e/'layout-after-court-identified.json').read_text())
combined={'binaries':prior['binaries']+courts['binaries']}
(e/'layout-after-frames-identified.json').write_text(json.dumps(combined,indent=2))
code=subprocess.call([sys.executable,str(e/'analyze_whole_frames.py'),str(e/'layout-after-frames-identified.json')])
if code:raise SystemExit(code)
print('All seven final-source artifacts censused; ordinary PE reserve unchanged.',reserves)
