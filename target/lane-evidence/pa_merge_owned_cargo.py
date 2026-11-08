import shutil,sys
from pathlib import Path
r=Path.cwd();sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from run_lane import child_environment
name,manifest,*args=sys.argv[1:];assert shutil.disk_usage('C:/').free>=25*1024**3
for index,argument in enumerate(args[:-1]):
 if argument=='--target-dir' and not Path(args[index+1]).is_absolute():
  args[index+1]=str(r/args[index+1])
mp=r/manifest;split=args.index('--') if '--' in args else len(args)
cmd=['cargo',*args[:split],'--manifest-path',str(mp),*args[split:]]
cwd=r if Path(manifest).parts[0]=='tools' else mp.parent
result=CargoRunner(r,application_build_environment(child_environment()))(cmd,cwd,r/'target/lane-evidence'/f'{name}.log')
raise SystemExit(result.returncode)
