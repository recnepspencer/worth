"""One serialized release diagnostic batch, no artifact-changing overrides."""
import json,shutil,sys,threading,time
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';out=e/'stack-peak-build';out.mkdir(exist_ok=True)
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from run_lane import child_environment
from logs import json_objects
runner=CargoRunner(r,application_build_environment(child_environment()))
stop=threading.Event();copied=set();dest=out/'coff';dest.mkdir(exist_ok=True)
consumer=r/'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml';bank=r/'workspaces/worth-query-bank-world/Cargo.toml'
prefixes=('invariant_reads','required_feature','shared_execution','worth_query_consumer_root','worth_query_bank_court_external_rail_server')
bases=[consumer.parent/'target/release',r/'target/release']
def observe():
 while not stop.is_set():
  for base in bases:
   dirs=[base/'deps',base/'examples']
   for directory in dirs:
    if not directory.exists():continue
    for p in directory.glob('*.rcgu.o'):
     if not p.name.startswith(prefixes) or p in copied:continue
     try:shutil.copyfile(p,dest/p.name);copied.add(p)
     except OSError:pass
  stop.wait(.15)
def disk():
 free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
 if free<25*1024**3:raise SystemExit(125)
t=threading.Thread(target=observe,daemon=True);t.start()
try:
 disk();o=out/'consumer';o.mkdir(exist_ok=True)
 result=runner(['cargo','build','--release','--manifest-path',str(consumer),'-p','worth-query-consumer-root','-p','worth-query-topology-entry','--bins','--examples','--message-format=json'],consumer.parent,o/'build.log')
 if result.returncode:raise SystemExit(result.returncode)
 expected={'worth-query-consumer-root','invariant_reads','required_feature','shared_execution'};artifacts={}
 for row in json_objects(result.stdout):
  if row.get('reason')=='compiler-artifact' and row.get('executable') and row['target']['name'] in expected:
   assert not row['profile']['test'];artifacts[row['target']['name']]={'exe':row['executable'],'cwd':str(Path(row['manifest_path']).parent),'crate':row['target']['name'].replace('-','_')}
 assert set(artifacts)==expected
 (out/'release-entries.json').write_text(json.dumps(artifacts,indent=2))
 disk();o=out/'bank';o.mkdir(exist_ok=True)
 reuse=build(bank,o/'build',runner,profile='release',selection={'packages':['worth-query-bank-court-external-rail-server'],'targets':{'test':['worth_query_bank_court_external_rail_server']}})
 names=json.loads((e/'head-closure-final-bank-plan.json').read_text())['names'];expr=' + '.join('test(='+n+')' for n in names);selected=[x for x in inventory(list_tests(reuse,o/'selection.json',runner,expr)) if x['selected']]
 assert len(selected)==2 and all(not x['ignored'] for x in selected)
 (o/'selection-audit.json').write_text(json.dumps(selected,indent=2))
 binaries=json.loads((o/'build/binaries.json').read_text())['rust-binaries'];b=[x for x in binaries.values() if x['binary-name']=='worth_query_bank_court_external_rail_server'];assert len(b)==1
 for name in names:artifacts[name]={'exe':b[0]['binary-path'],'cwd':str(bank.parent/'crates/worth-query-bank-court-external-rail-server'),'crate':'worth_query_bank_court_external_rail_server','test':name}
 (out/'release-entries.json').write_text(json.dumps(artifacts,indent=2))
 print('All6release entries built; copied entry COFF objects',len(copied),flush=True)
finally:stop.set();t.join()
