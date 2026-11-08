"""Refresh only the changed Root court; reuse six hash-identical scan records."""
import gc,hashlib,json,struct,subprocess,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';sys.path.insert(0,str(e))
from lifecycle_compiled_frames import Frames
prior=json.loads((e/'layout-after-frames-identified.json').read_text());prior['binaries']=[x for x in prior['binaries'] if x['binary']!='root-court']
assert len(prior['binaries'])==6
manifest={x['binary']:x['exe'] for x in prior['binaries']}
for x in prior['binaries']:
 a=x['artifact_identity']
 for path,digest in [(x['exe'],a['exe_sha256']),(a['pdb_path'],a['pdb_sha256'])]:
  with open(path,'rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==digest,path
b=json.loads((e/'layout-topology-last/bridge-court/build/binaries.json').read_text())['rust-binaries'];root=[x['binary-path'] for x in b.values() if x['binary-name']=='worth_court_runtime_bridge'];assert len(root)==1
manifest['root-court']=root[0];(e/'layout-final-frame-inputs.json').write_text(json.dumps(manifest,indent=2))
reserves={}
for n in ['invariant_reads','required_feature','shared_execution','worth-query-consumer-root']:
 pe=Frames(manifest[n]);reserves[n]=struct.unpack_from('<Q',pe.data,pe.u32(0x3c)+24+72)[0];del pe;gc.collect()
assert set(reserves.values())=={1048576}
(e/'layout-final-main-stack-reserves.json').write_text(json.dumps(reserves,indent=2))
(e/'layout-final-root-inputs.json').write_text(json.dumps({'root-court':root[0]},indent=2))
for cmd in [['whole_binary_frames.py','layout-final-root-inputs.json','layout-final-root-frames.json'],['pdb_public_frames.py','layout-final-root-frames.json','layout-final-root-publics.json'],['coff_frame_attribution.py','layout-final-root-publics.json','layout-final-root-identified.json']]:
 code=subprocess.call([sys.executable,*[str(e/x) for x in cmd]])
 if code:raise SystemExit(code)
new=json.loads((e/'layout-final-root-identified.json').read_text());(e/'layout-final-frames-identified.json').write_text(json.dumps({'binaries':prior['binaries']+new['binaries']},indent=2))
code=subprocess.call([sys.executable,str(e/'analyze_whole_frames.py'),str(e/'layout-final-frames-identified.json')]);assert code==0
print('Six unchanged artifact/PDB hashes verified; one changed Root court refreshed. Stack reserves',reserves)
