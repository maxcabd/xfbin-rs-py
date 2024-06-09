use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big)]
#[br(import_raw(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkCamera {
    #[brw(ignore)]
    pub version: u16,

    #[brw(pad_before = 4)]
    pub fov: f32,
}

impl NuccChunk for NuccChunkCamera {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkCamera
    }

    fn version(&self) -> u16 {
        self.version
    }
}