"""Diagnostic only: unchanged entries,64MiB worker, painting + actual-SP traps."""
import hashlib,json,os,sys
from pathlib import Path
from symbols import Symbols
from debug_session import Session
r=Path(__file__).resolve().parents[3];e=r/'target/lane-evidence';out=e/'stack-peak-runs';out.mkdir(exist_ok=True)
profile,name=sys.argv[1:]
for key in ['RUST_MIN_STACK','WORTH_BANK_ASSESSMENT_WIDTH','WORTH_BANK_HISTORY_WIDTH','WORTH_BANK_PRESENT_DEADLINE']:os.environ.pop(key,None)
os.environ.setdefault('WORTH_REPO',str(r));os.environ['_NO_DEBUG_HEAP']='1'
if profile=='dev':
 original=json.loads((e/'layout-r3/ordinary/actual-entry-artifacts.json').read_text());entries={k:{'exe':v['exe'],'cwd':v['cwd'],'crate':k.replace('-','_')} for k,v in original.items()}
 meta=json.loads((e/'layout-r3/bank/build/binaries.json').read_text())['rust-binaries'];bank=[v for v in meta.values() if v['binary-name']=='worth_query_bank_court_external_rail_server'];assert len(bank)==1
 for n in json.loads((e/'head-closure-final-bank-plan.json').read_text())['names']:entries[n]={'exe':bank[0]['binary-path'],'cwd':str(r/'workspaces/worth-query-bank-world/crates/worth-query-bank-court-external-rail-server'),'crate':'worth_query_bank_court_external_rail_server','test':n}
else:
 assert profile=='release';entries=json.loads((e/'stack-peak-build/release-entries.json').read_text())
if name=='assessment':name=next(k for k in entries if k.endswith('assessment_history_settles_at_each_request_width'))
if name=='transfer':name=next(k for k in entries if k.endswith('seeded_transfers_match_at_host_policy_and_each_request_width'))
a=entries[name];key=profile+'-'+('assessment' if 'assessment_history_settles' in name else 'transfer' if 'seeded_transfers_match' in name else name)
body=a.get('test','run' if name=='worth-query-consumer-root' else 'main');a['measured_body']=body
symbols=Symbols(a['exe'],a['crate']+'::'+body)
with open(a['exe'],'rb') as f:before=hashlib.file_digest(f,'sha256').hexdigest()
print('EXACT ENTRY RVAS',symbols.entries,'PROFILE',profile,'SHA256',before,flush=True)
paint_file=out/(key+'-paint.json')
if os.environ.get('STACK_PEAK_REUSE_PAINT')=='1' and paint_file.exists(): first=json.loads(paint_file.read_text())
else:
 first=Session(a,symbols).launch();paint_file.write_text(json.dumps(first,indent=2))
second=Session(a,symbols,first).launch();(out/(key+'-path.json')).write_text(json.dumps(second,indent=2))
assert all(x['stable'] for x in second),'painting high-water changed between passes'
with open(a['exe'],'rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==before,'disk artifact was modified'
result={'profile':profile,'entry':name,'artifact':a,'exe_sha256':before,'peak_bytes':max(x['peak_bytes'] for x in second),'fits_1MiB':max(x['peak_bytes'] for x in second)<=1024**2,'processes':second,'diagnostic_only':True,'paint_resolution_bytes':8}
(out/(key+'.json')).write_text(json.dumps(result,indent=2));print('RESULT',key,result['peak_bytes'],'fits1MiB',result['fits_1MiB'],flush=True)
