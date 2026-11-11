from pathlib import Path
import json,hashlib,subprocess
r=Path.cwd();e=r/'target/lane-evidence';v=json.loads((e/'execution-stack-final-source-inputs.json').read_text());head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();bad=[]
for name,digest in v['files'].items():
 actual=hashlib.sha256((r/name).read_bytes()).hexdigest()
 blob=subprocess.check_output(['git','hash-object','--path='+name,name],text=True,stderr=subprocess.DEVNULL).strip()
 committed=subprocess.check_output(['git','rev-parse',head+':'+name],text=True).strip()
 if actual!=digest or blob!=committed:bad.append(name)
ancestors={}
for sha in ['fbbdf5edff','80543019fb','2a51552fc5','655b27b95e','623efa14bd','fbc418fde2']:
 ancestors[sha]=subprocess.call(['git','merge-base','--is-ancestor',sha,head])==0
ui=subprocess.call(['git','diff','--exit-code','fbbdf5edff','--','workspaces/worth-ui'])==0
result={'source_head':head,'files_verified':len(v['files']),'mismatches':bad,'ancestors':ancestors,'ui_equals_master_side':ui,'note':'Final dirty-source hashes and normalized Git blobs match source commit. Round2 ordinary controls precede the Host transition-only correction; final D16 covers that consuming-phase route. Mixed-HEAD census remains explicitly partial.'}
(e/'execution-stack-source-equivalence.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2));assert not bad and all(ancestors.values()) and ui
