"""Run both audited bounded Bank parents on the default harness stack, without Cargo."""
import json,os,subprocess,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';base=e/'layout-r3/bank';build=base/'build'
plan=json.loads((e/'head-closure-final-bank-plan.json').read_text());names=plan['names']
rows=json.loads((base/'bank-parent-selection-audit.json').read_text());assert all(any(row['test']==name and not row['ignored'] for row in rows) for name in names)
binaries=json.loads((build/'binaries.json').read_text())['rust-binaries']
choices=[row for row in binaries.values() if row['binary-name']=='worth_query_bank_court_external_rail_server'];assert len(choices)==1
binary=choices[0];metadata=json.loads((build/'cargo-metadata.json').read_text());package=next(row for row in metadata['packages'] if row['id']==binary['package-id']);cwd=Path(package['manifest_path']).parent
env=os.environ.copy()
for key in ['RUST_MIN_STACK','WORTH_BANK_ASSESSMENT_WIDTH','WORTH_BANK_HISTORY_WIDTH']:env.pop(key,None)
env.setdefault('WORTH_REPO',str(r))
print('RUST_MIN_STACK present:', 'RUST_MIN_STACK' in env,flush=True)
print('AUDITED BUILD:',str(build),flush=True)
listed=subprocess.run([binary['binary-path'],'--list','--exact',*names],cwd=cwd,env=env,capture_output=True,text=True)
assert listed.returncode==0 and all(name+': test' in listed.stdout.splitlines() for name in names),listed.stdout+listed.stderr
command=[binary['binary-path'],'--exact',*names,'--nocapture']
print('DIRECT COMMAND:',json.dumps(command),flush=True)
raise SystemExit(subprocess.run(command,cwd=cwd,env=env).returncode)
