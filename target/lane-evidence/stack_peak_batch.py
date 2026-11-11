import subprocess,sys,json,time
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';rows=[]
for profile in ['dev','release']:
 for name in ['invariant_reads','required_feature','shared_execution','worth-query-consumer-root','assessment','transfer']:
  key=profile+'-'+name;result=e/'stack-peak-runs'/(key+'.json')
  if result.exists():
   rows.append([key,'reuse']);continue
  code=subprocess.call([sys.executable,str(e/'run_evidence.py'),'stack-peak-'+key,sys.executable,str(e/'stack_peak/measure.py'),profile,name])
  rows.append([key,code]);(e/'stack-peak-batch-results.json').write_text(json.dumps(rows,indent=2))
  if code:break
 else:continue
 break
print(json.dumps(rows),flush=True)
sys.exit(int(any(isinstance(code,int) and code for key,code in rows)))
