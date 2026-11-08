"""A stack-free, register-preserving minimum-only stack-probe recorder."""
import struct
from windows_debug import *
class ProbeRecorder:
 def __init__(self,process,probe,low,high):
  self.process=process;self.probe=probe;self.original=read(process,probe,8)
  assert self.original==bytes.fromhex('48 83 ec 10 4c 89 14 24'),self.original.hex()
  self.base=0
  for delta in (0x10000000,-0x10000000,0x20000000,-0x20000000,0x30000000,-0x30000000):
   hint=(probe+delta)&~65535
   self.base=k.VirtualAllocEx(process,hint,4096,0x3000,0x40)
   if self.base:break
  assert self.base,'no nearby diagnostic recorder page'
  self.code=bytearray();self.labels={};self.fix=[];self.data=0x300
  def emit(x):self.code.extend(bytes.fromhex(x))
  def mem(op,slot):
   emit(op);self.code.extend(struct.pack('<i',self.data+slot-(len(self.code)+4)))
  def branch(op,label):emit(op);self.fix.append((len(self.code),label));self.code.extend(b'\0'*4)
  mem('48 3b 25',0);branch('0f 82','prefix') # below measured stack
  mem('48 3b 25',8);branch('0f 83','prefix') # above measured stack
  mem('4c 89 15',24);mem('4c 89 1d',32) # save R10/R11 only for this stack
  emit('4c 8d 54 24 08');emit('4c 2b d0') # exact caller SP after allocation
  mem('4c 3b 15',16);branch('0f 83','restore')
  mem('4c 89 15',16)
  mem('4c 8b 15',24);mem('4c 8b 1d',32)
  self.notify=self.base+len(self.code);emit('cc');branch('e9','prefix')
  self.labels['restore']=len(self.code);mem('4c 8b 15',24);mem('4c 8b 1d',32)
  self.labels['prefix']=len(self.code);self.code.extend(self.original)
  emit('e9');self.code.extend(struct.pack('<i',probe+8-(self.base+len(self.code)+4)))
  for offset,label in self.fix:struct.pack_into('<i',self.code,offset,self.labels[label]-(offset+4))
  assert len(self.code)<self.data
  write(process,self.base,bytes(self.code));write(process,self.base+self.data,struct.pack('<5Q',low,high,high,0,0))
  # The copied original SUB overwrites arithmetic flags before the real helper
  # reads them. No observer instruction changes SP, XMM, RAX or argument regs.
  patch=b'\xe9'+struct.pack('<i',self.base-(probe+5))+b'\x90'*3
  bp(process,probe,patch)
 def restore(self):
  bp(self.process,self.probe,self.original)
  # Retain this page until process exit: another thread may be in the bypass.
