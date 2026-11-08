"""A diagnostic worker executes unchanged machine code on a painted 64MiB stack."""
import ctypes as c,struct
from windows_debug import *
from probe_recorder import ProbeRecorder
STACK=64*1024*1024
class Worker:
 def __init__(self,process,original,origctx,entry,thread,tid,index,symbols,base,target=None):
  self.process=process;self.original=original;self.origctx=origctx;self.entry=entry;self.thread=thread;self.tid=tid;self.index=index;self.symbols=symbols;self.base=base;self.target=target;self.phase='entry';self.probe_peak=0;self.probes=0;self.path=[];self.peak_sp=0;self.post=None;self.watch_hits=0
 def start(self,ctx):
  self.saved=c.create_string_buffer(ctx.raw,1232);self.return_pc=struct.unpack('<Q',read(self.process,get(ctx,'rsp'),8))[0]
  self.scratch=checked(k.VirtualAllocEx(self.process,None,4096,0x3000,4));self.entry_sp=get(ctx,'rsp')
  api=c.cast(k.GetCurrentThreadStackLimits,P).value
  assert read(self.process,api,16)==c.string_at(api,16),'remote stack-limit API mapping differs'
  sp=self.entry_sp-0x30;write(self.process,sp,struct.pack('<Q',self.entry))
  setreg(ctx,'rsp',sp);setreg(ctx,'rip',api);setreg(ctx,'rcx',self.scratch);setreg(ctx,'rdx',self.scratch+8)
  checked(k.SetThreadContext(self.thread,ctx));self.phase='limits'
 def paint(self):
  low,self.high=struct.unpack('<QQ',read(self.process,self.scratch,16));allocation=query(self.process,self.high-1).allocation_base
  assert self.high-allocation==STACK,(hex(low),hex(self.high),hex(allocation))
  self.low=allocation+4096;size=self.high-self.low;checked(k.VirtualAllocEx(self.process,self.low,size,0x1000,4));checked(k.VirtualProtectEx(self.process,self.low,size,4,c.byref(w.DWORD())))
  # Manual commitment must publish the same NT_TIB bound that OS growth would.
  # Otherwise native heap/unwind validation sees the active SP outside its stack.
  tib_base,tib_limit=struct.unpack('<QQ',read(self.process,self.teb+8,16));assert tib_base==self.high
  write(self.process,self.teb+16,struct.pack('<Q',self.low))
  print('STACK_LIMITS',self.index,hex(low),hex(self.high),'committed',hex(self.low),flush=True)
  pattern=b'\xa5'*(1024*1024)
  for address in range(self.low,self.entry_sp,len(pattern)):write(self.process,address,pattern[:min(len(pattern),self.entry_sp-address)])
  ctx=self.saved;setreg(ctx,'rip',self.entry);setreg(ctx,'dr1',self.return_pc);setreg(ctx,'dr2',0);dr7=4
  self.hook=ProbeRecorder(self.process,self.base+self.symbols.entries['probe'],self.low,self.high)
  if self.target and self.target['kind']=='write':
   target_depth=self.target['peak_bytes']-self.target['limits']['entry_baseline']
   setreg(ctx,'dr0',self.entry_sp-target_depth);dr7|=1|(1<<16)|(2<<18)
  setreg(ctx,'dr7',dr7);checked(k.SetThreadContext(self.thread,ctx));checked(k.VirtualFreeEx(self.process,self.scratch,0,0x8000));self.phase='body'
  self.api_limits={'low':low,'high':self.high,'reserve_bytes':STACK,'entry_baseline':self.high-self.entry_sp}
 def event(self,ctx):
  reasons=get(ctx,'dr6')
  if reasons&2:return self.finish(ctx)
  if reasons&8:
   depth=self.high-get(ctx,'rsp')
   if depth>=self.peak_sp:
    self.peak_sp=depth
    if not self.target or self.target['kind']=='probe':self.path=self.symbols.capture(self.process,self.thread,ctx,self.base)
   setreg(ctx,'dr3',0);setreg(ctx,'dr7',get(ctx,'dr7')&~64)
  if reasons&1:
   self.watch_hits+=1
   self.path=self.symbols.capture(self.process,self.thread,ctx,self.base)
   # One exact low-water access is sufficient; avoid perturbing timed work.
   setreg(ctx,'dr7',get(ctx,'dr7')&~1)
  setreg(ctx,'dr6',0);resume_flag(ctx);checked(k.SetThreadContext(self.thread,ctx));return None
 def new_minimum(self,ctx):
  self.probes+=1;
  request=get(ctx,'rax');candidate=self.high-(get(ctx,'rsp')+8-request)
  if candidate>self.probe_peak:
   self.probe_peak=candidate;ret=struct.unpack('<Q',read(self.process,get(ctx,'rsp'),8))[0]
   instruction=read(self.process,ret,3)
   assert instruction==b'\x48\x29\xc4',('unknown stack-allocation instruction',instruction.hex(),hex(ret))
   self.post=ret+3;setreg(ctx,'dr3',self.post);setreg(ctx,'dr7',get(ctx,'dr7')|64)
  checked(k.SetThreadContext(self.thread,ctx))
 def finish(self,ctx):
  written=None;chunk=1024*1024
  for address in range(self.low,self.entry_sp,chunk):
   data=read(self.process,address,min(chunk,self.entry_sp-address));remaining=data.lstrip(b'\xa5')
   if remaining:
    touched=address+len(data)-len(remaining);written=self.high-(touched&~7);break
  written=written or self.high-self.entry_sp
  peak=max(written,self.peak_sp);kind='write' if written>=self.peak_sp else 'probe'
  result={'index':self.index,'peak_bytes':peak,'written_high_water':written,'allocated_rsp_high_water':self.peak_sp,'kind':kind,'limits':self.api_limits,'probes':self.probes,'watch_hits':self.watch_hits,'call_path':self.path}
  if self.target:
   result['target_peak']=self.target['peak_bytes'];result['stable']=(peak-(self.high-self.entry_sp)==self.target['peak_bytes']-self.target['limits']['entry_baseline'])
   if self.target['kind']=='write':assert self.watch_hits,'low-water slot was never touched in path pass'
  self.hook.restore()
  for name in ['rax','rdx']:setreg(self.origctx,name,get(ctx,name))
  original_sp=get(self.origctx,'rsp');setreg(self.origctx,'rip',struct.unpack('<Q',read(self.process,original_sp,8))[0]);setreg(self.origctx,'rsp',original_sp+8)
  checked(k.SetThreadContext(self.original,self.origctx));assert k.ResumeThread(self.original)!=0xffffffff
  setreg(ctx,'dr7',0);setreg(ctx,'dr6',0);checked(k.SetThreadContext(self.thread,ctx));self.phase='done';return result
