#!/usr/bin/env python3
"""Package source and verified artifacts; exclude build caches and private signing material."""
import pathlib,zipfile,hashlib,sys
root=pathlib.Path(__file__).resolve().parents[1]
out=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else root.parent/'PROPHET-XII-DELIVERY.zip')
files=[]
for p in root.rglob('*'):
 if not p.is_file():continue
 rel=p.relative_to(root)
 if 'node_modules' in rel.parts or rel.parts[0] in ['target','.git']:continue
 if rel.parts[:2]==('android','build'):continue
 if p.name in ['core','.DS_Store','ARTIFACTS-SHA256SUMS'] or p.suffix in ['.key','.p12','.jks','.keystore']:continue
 files.append(p)
manifest=root/'ARTIFACTS-SHA256SUMS'
manifest.write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.relative_to(root).as_posix()+'\n'for p in sorted(files)))
files.append(manifest)
with zipfile.ZipFile(out,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6,allowZip64=True)as z:
 for p in sorted(files):z.write(p,arcname='prophet-xii/'+p.relative_to(root).as_posix(),compress_type=zipfile.ZIP_STORED if p.name.endswith('.tar.gz')else zipfile.ZIP_DEFLATED)
with zipfile.ZipFile(out)as z:
 assert z.testzip()is None
 assert any(n.endswith('/prophet-standalone')for n in z.namelist())
 assert not any('/node_modules/'in n or '/target/'in n or '/private-signing/'in n for n in z.namelist())
digest=hashlib.sha256(out.read_bytes()).hexdigest()
out.with_suffix('.zip.sha256').write_text(digest+'  '+out.name+'\n')
print(f'{out}\n{out.stat().st_size} bytes\nSHA256 {digest}\n{len(files)} files')
