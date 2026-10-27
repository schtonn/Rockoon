//! Bounded reader for the three original render meshes in Balls.nmo (Virtools 2.1).
//! Format references: LibCmo CKFileReader, CKStateChunkOthers and CKMesh, MIT; see docs.
use super::math::V;
use flate2::read::ZlibDecoder;
use std::{io::Read, path::Path};
#[derive(Debug)]
pub struct Mesh {
    pub positions: Vec<V>,
    pub normals: Vec<V>,
    pub uv: Vec<[f32; 2]>,
    pub faces: Vec<[usize; 3]>,
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let b = self.bytes.get(self.pos..end)?;
        self.pos = end;
        Some(b)
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn f32(&mut self) -> Option<f32> {
        let n = f32::from_bits(self.u32()?);
        n.is_finite().then_some(n)
    }
    fn v(&mut self) -> Option<V> {
        Some(V(self.f32()?, self.f32()?, self.f32()?))
    }
}
fn unpack(bytes: &[u8], size: usize) -> Option<Vec<u8>> {
    if size > 16 * 1024 * 1024 {
        return None;
    }
    if bytes.len() == size {
        return Some(bytes.to_vec());
    }
    let mut out = Vec::new();
    ZlibDecoder::new(bytes)
        .take(size as u64 + 1)
        .read_to_end(&mut out)
        .ok()?;
    (out.len() == size).then_some(out)
}
fn section(bytes: &[u8], id: u32) -> Option<&[u8]> {
    // Latest CKStateChunk format: version 7, data size in DWORDs, linked identifiers.
    if *bytes.get(2)? != 7 || *bytes.get(1)? != 32 {
        return None;
    }
    let mut chunk = Reader::new(bytes);
    chunk.u32()?;
    let size = chunk.u32()? as usize;
    let data = chunk.take(size.checked_mul(4)?)?;
    let mut offset = 0;
    for _ in 0..64 {
        let mut r = Reader::new(data.get(offset..)?);
        let tag = r.u32()?;
        let next = r.u32()? as usize;
        let end = if next == 0 {
            data.len()
        } else {
            next.checked_mul(4)?
        };
        if end <= offset || end > data.len() {
            return None;
        }
        if tag == id {
            return data.get(offset + 8..end);
        }
        if next == 0 {
            return None;
        }
        offset = end;
    }
    None
}
impl Mesh {
    fn read(chunk: &[u8]) -> Option<Self> {
        let mut r = Reader::new(section(chunk, 0x20000)?);
        let count = r.u32()? as usize;
        if count == 0 || count > 4096 {
            return None;
        }
        let flags = r.u32()?;
        let words = r.u32()? as usize;
        if flags & 0x14 != 0 || words == 0 {
            return None;
        } // Require source positions and normals.
        let mut r = Reader::new(r.take((words - 1).checked_mul(4)?)?);
        let positions = (0..count).map(|_| r.v()).collect::<Option<Vec<_>>>()?;
        r.take(if flags & 1 != 0 { 4 } else { 4 * count })?;
        r.take(if flags & 2 != 0 { 4 } else { 4 * count })?;
        let normals = (0..count).map(|_| r.v()).collect::<Option<Vec<_>>>()?;
        let uv = if flags & 8 != 0 {
            vec![[r.f32()?, r.f32()?]; count]
        } else {
            (0..count)
                .map(|_| Some([r.f32()?, r.f32()?]))
                .collect::<Option<Vec<_>>>()?
        };
        if positions.iter().any(|v| v.length() > 3.0) {
            return None;
        }
        let mut r = Reader::new(section(chunk, 0x10000)?);
        let count_faces = r.u32()? as usize;
        if count_faces > 8192 {
            return None;
        }
        let faces = (0..count_faces)
            .map(|_| {
                let b = r.take(8)?;
                let mut face = [0; 3];
                for i in 0..3 {
                    face[i] = u16::from_le_bytes([b[i * 2], b[i * 2 + 1]]) as usize;
                    if face[i] >= count {
                        return None;
                    }
                }
                Some(face)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            positions,
            normals,
            uv,
            faces,
        })
    }
}
pub fn load(root: &Path) -> Option<[Mesh; 3]> {
    let path = root.join("3D Entities/Balls.nmo");
    if std::fs::metadata(&path).ok()?.len() > 4 * 1024 * 1024 {
        return None;
    }
    parse(&std::fs::read(path).ok()?)
}
fn parse(bytes: &[u8]) -> Option<[Mesh; 3]> {
    let mut r = Reader::new(bytes);
    if r.take(8)? != b"Nemo Fi\0" {
        return None;
    }
    let mut h = [0; 14];
    for n in &mut h {
        *n = r.u32()? as usize;
    }
    if h[2] != 8 || h[8] > 64 || h[9] > 10000 {
        return None;
    }
    let header = unpack(r.take(h[5])?, h[13])?;
    let body = unpack(r.take(h[6])?, h[7])?;
    let mut r = Reader::new(&header);
    let mut names = Vec::new();
    for _ in 0..h[9] {
        r.take(12)?;
        let n = r.u32()? as usize;
        names.push(r.take(n)?);
    }
    let mut r = Reader::new(&body);
    for _ in 0..h[8] {
        r.take(8)?;
        let n = r.u32()? as usize;
        r.take(n)?;
    }
    let mut found: [Option<Mesh>; 3] = [None, None, None];
    for name in names {
        let n = r.u32()? as usize;
        let chunk = r.take(n)?;
        let name = name.strip_suffix(&[0]).unwrap_or(name);
        let index = match name {
            b"Ball_Wood_Mesh" => 0,
            b"Ball_Stone_HighRes_Mesh" => 1,
            b"Ball_Paper_Mesh" => 2,
            _ => continue,
        };
        found[index] = Some(Mesh::read(chunk)?);
    }
    let [a, b, c] = found;
    Some([a?, b?, c?])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_containers_are_rejected() {
        assert!(parse(b"Nemo Fi\0").is_none());
        let mut b = vec![0u8; 64];
        b[..8].copy_from_slice(b"Nemo Fi\0");
        b[16] = 8;
        b[28..32].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse(&b).is_none());
        assert!(unpack(&[0; 20], 17 * 1024 * 1024).is_none());
    }
}
