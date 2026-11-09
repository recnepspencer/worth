from pathlib import Path
import json,subprocess
r=Path.cwd();e=r/'target/lane-evidence';peer=r.parent/'m3-p3-lifecycle'
prefix='workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/provider/idempotency/snapshot_resolution/original_receipt/outbox/'
rows=json.loads((e/'h8-query-grants.json').read_text(encoding='utf-8'))
for suffix in ['tests.rs','tests/installation.rs']:
 path=prefix+suffix;diff=['git','-C',str(peer),'diff','--quiet','e787b51b3e','--',path];status=['git','-C',str(peer),'status','--porcelain','--',path]
 d=subprocess.run(diff);s=subprocess.check_output(status,text=True);row={'path':path,'peer':str(peer),'peer_HEAD':subprocess.check_output(['git','-C',str(peer),'rev-parse','HEAD'],text=True).strip(),'diff_command':diff,'diff_exit':d.returncode,'status_command':status,'status':s,'clear_of_PC':d.returncode==0 and not s,'new_file':True};print(json.dumps(row),flush=True);assert row['clear_of_PC'];rows.append(row)
(e/'h8-query-grants.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
