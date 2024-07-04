//! # nuccAnm
//! nuccAnm is a chunk that contains animation data.
//! The extension ".anm" stands for "Animation".
//!
use std::{default, io::{Read, Seek, Write}};
use pyo3::prelude::*;
use binrw::{binrw, BinRead, BinReaderExt, BinResult, BinWrite, ReadOptions, WriteOptions};

use super::{NuccChunk, NuccChunkType};

#[pyclass]
#[binrw]
#[brw(big)]
#[br(import_raw(version: u16))]
#[derive(Debug, Clone, Default)]
pub struct NuccChunkAnm {
    #[brw(ignore)]
    pub version: u16,

    pub frame_count: u32,

    #[bw(calc = 100)]
    pub frame_size: u32,

    #[bw(calc = entries.len() as u16)]
    pub entry_count: u16,

    pub is_looped: u16,

    #[bw(calc = clumps.len() as u16)]
    pub clump_count: u16,

    #[bw(calc = other_entries_indices.len() as u16)]
    pub other_entry_count: u16,

    #[bw(calc = unk_entry_indices.len() as u16)]
    pub unk_entry_count: u16,

    #[bw(calc = coord_parents.len() as u16)]
    pub coord_count: u16,

    #[br(count = clump_count)]
    pub clumps: Vec<AnmClump>,

    #[br(count = other_entry_count)]
    pub other_entries_indices: Vec<u32>,

    #[br(count = unk_entry_count)]
    pub unk_entry_indices: Vec<u32>,

    #[br(count = coord_count)]
    pub coord_parents: Vec<CoordParent>,

    #[br(count = entry_count)]
    pub entries: Vec<AnmEntry>,
}

#[pyclass]
#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmClump {

    #[pyo3(get, set)]
    pub clump_index: u32,

    #[bw(calc = bone_material_indices.len() as u16)]
    pub bone_material_count: u16,

    #[bw(calc = model_indices.len() as u16)]
    pub model_count: u16,

    #[pyo3(get, set)]
    #[br(count = bone_material_count)]
    pub bone_material_indices: Vec<u32>,

    #[pyo3(get, set)]
    #[br(count = model_count)]
    pub model_indices: Vec<u32>,
}

#[pymethods]
impl AnmClump {
    #[new]
    #[pyo3(signature = (clump_index = 0, bone_material_indices = None, model_indices = None))]
    pub fn __new__(
        clump_index: u32,
        bone_material_indices: Option<Vec<u32>>,
        model_indices: Option<Vec<u32>>,
    ) -> Self {
        Self {
            clump_index,
            bone_material_indices: bone_material_indices.unwrap_or_default(),
            model_indices: model_indices.unwrap_or_default(),
        }
    }
}



#[pyclass]
#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct CoordParent {
    pub parent: AnmCoord,
    pub child: AnmCoord,
}

#[pymethods]
impl CoordParent {
    #[new]
    #[pyo3(signature = (parent = AnmCoord::__new__(0, 0), child = AnmCoord::__new__(0, 0)))]
    pub fn __new__(parent: AnmCoord, child: AnmCoord) -> Self {
        Self { parent, child }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "CoordParent(parent={:?}, child={:?})",
            self.parent, self.child
        ))
    }
    

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
    
}


#[pyclass]
#[binrw]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnmCoord {
    pub clump_index: i16,
    pub coord_index: u16,
}

#[pymethods]
impl AnmCoord {
    #[new]
    #[pyo3(signature = (clump_index = 0, coord_index = 0))]
    pub fn __new__(clump_index: i16, coord_index: u16) -> Self {
        Self {
            clump_index,
            coord_index,
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "AnmCoord(clump_index={}, coord_index={})",
            self.clump_index, self.coord_index
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

#[pyclass]
#[binrw]
#[brw(repr(u16))]
#[derive(Debug, Clone, Default)]
pub enum EntryFormat {
    Coord = 1,
    Camera = 2,
    Material = 4,
    LightDirc = 5,
    LightPoint = 6,
    Ambient = 8,
    MorphModel = 9,

    #[default]
    Unknown,
}

#[pyclass]
#[binrw]
#[derive(Debug, Clone)]
pub struct AnmEntry {
    #[pyo3(get, set)]
    pub coord: AnmCoord,

    #[pyo3(get, set)]
    pub entry_format: EntryFormat,

    #[bw(calc = track_headers.len() as u16)]
    pub track_count: u16,

    #[br(count = track_count)]
    pub track_headers: Vec<TrackHeader>,

    #[br(parse_with = read_tracks(track_headers.iter()))]
    #[br(align_after = 4)]
    pub tracks: Vec<AnmTrack>,
}
 

#[pyclass]
#[binrw]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackHeader {
    #[pyo3(get, set)]
    pub track_index: u16,
    #[pyo3(get, set)]
    pub key_format: NuccAnmKeyFormat,
    #[pyo3(get, set)]
    pub frame_count: u16,

    #[bw(calc = key_format.size_per_frame() as u16 * frame_count)]
    pub track_size: u16,
}

#[pymethods]
impl TrackHeader {
    #[new]
    #[pyo3(signature = (track_index = 0, key_format = None, frame_count = 0))]
    pub fn __new__(
        track_index: u16,
        key_format: Option<NuccAnmKeyFormat>,
        frame_count: u16,
    ) -> Self {
        Self {
            track_index,
            key_format: key_format.unwrap_or_default(),
            frame_count,
        }
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "TrackHeader(track_index={}, key_format={:?}, frame_count={})",
            self.track_index, self.key_format, self.frame_count
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

#[pyclass]
#[binrw]
#[brw(repr(u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NuccAnmKeyFormat {
    Vector3Fixed = 0x5,
    Vector3Linear = 0x6,
    Vector3Bezier = 0x7,
    EulerXYZFixed  = 0x8,
    EulerInterpolated = 0x9,
    QuaternionLinear = 0xA,
    FloatFixed = 0xB,
    FloatLinear = 0xC,
    Vector2Fixed = 0xD,
    Vector2Linear = 0xE,
    OpacityShortTable = 0xF,
    ScaleShortTable = 0x10,
    QuaternionShortTable = 0x11,
    ColorRGBTable = 0x14,
    Vector3Table = 0x15,
    FloatTable = 0x16,
    QuaternionTable = 0x17,
    FloatTableNoInterp = 0x18,
    Vector3ShortLinear = 0x19,
    Vector3TableNoInterp = 0x1A,
    QuaternionShortTableNoInterp = 0x1B,
    OpacityShortTableNoInterp = 0x1D,

    #[default]
    Unknown,
}

impl NuccAnmKeyFormat {
    pub fn size_per_frame(&self) -> usize {
        match self {
            NuccAnmKeyFormat::OpacityShortTable | NuccAnmKeyFormat::OpacityShortTableNoInterp => 0x02,
            NuccAnmKeyFormat::ColorRGBTable => 0x03,
            NuccAnmKeyFormat::FloatFixed | NuccAnmKeyFormat::FloatTable | NuccAnmKeyFormat::FloatTableNoInterp => {
                0x04
            }
            NuccAnmKeyFormat::ScaleShortTable => 0x06,
            NuccAnmKeyFormat::FloatLinear
            | NuccAnmKeyFormat::Vector2Fixed
            | NuccAnmKeyFormat::QuaternionShortTable
            | NuccAnmKeyFormat::QuaternionShortTableNoInterp => 0x08,
            NuccAnmKeyFormat::Vector3Fixed
            | NuccAnmKeyFormat::EulerXYZFixed
            | NuccAnmKeyFormat::Vector2Linear
            | NuccAnmKeyFormat::Vector3Table
            | NuccAnmKeyFormat::Vector3TableNoInterp => 0x0C,
            NuccAnmKeyFormat::Vector3Linear | NuccAnmKeyFormat::QuaternionTable => 0x10,
            NuccAnmKeyFormat::QuaternionLinear => 0x14,
            NuccAnmKeyFormat::Vector3Bezier => 0x10,
            NuccAnmKeyFormat::EulerInterpolated => 0x0C,
            NuccAnmKeyFormat::Vector3ShortLinear => 0x0C,
            NuccAnmKeyFormat::Unknown => 0x0C,
        }
    }

    
}

#[pyclass]
#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub enum NuccAnmKey {

    Vec3 { values: (f32, f32, f32) },
    Vec3Linear { frame: i32, values: (f32, f32, f32) },
    Vec4Linear { frame: i32, values: (f32, f32, f32, f32) },
    Float { values: f32 },
    FloatLinear { frame: i32, values: f32 },
    I16Vec { values: i16 },
    I16Vec3 { values: (i16, i16, i16) },
    ShortVec4 { values: (i16, i16, i16, i16) }, // QuaternionShortTable
    Color { values: (u8, u8, u8) }, // ColorRGBTable

    // Add more variants as needed...
    Unknown {}
    
}

impl default::Default for NuccAnmKey {
    fn default() -> Self {
        NuccAnmKey::Unknown {}
    }
}

#[pyclass]
#[binrw]
#[br(import_raw(header: TrackHeader))]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmTrack {
    #[pyo3(get, set)]
    #[br(parse_with = |r, o, _h: TrackHeader| read_track_data(r, o, header))]
    pub keys: Vec<NuccAnmKey>,
}

#[pymethods]
impl AnmTrack {
    #[new]
    #[pyo3(signature = (keys = None))]
    pub fn __new__(
        keys: Option<Vec<NuccAnmKey>>,
    ) -> Self {
        Self {
            keys: keys.unwrap_or_default(),
        }
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "NuccAnmKeyFormat {{keyframes: {:?}}}",
            self.keys
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

fn read_tracks<'it, R, T, Arg, Ret, It>(
    it: It,
) -> impl FnOnce(&mut R, &ReadOptions, ()) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    R: Read + Seek,
    Arg: Clone + 'static,
    Ret: FromIterator<T> + 'static,
    It: Iterator<Item = &'it Arg> + 'it,
{
    move |reader, options, _| {
        it.map(|arg| T::read_options(reader, options, arg.clone()))
            .collect()
    }
}


fn read_track_data<R: Read + Seek>(
    reader: &mut R,
    _: &ReadOptions, 
    header: TrackHeader

) -> BinResult<Vec<NuccAnmKey>> {
    
    let mut keyframes: Vec<NuccAnmKey> = Vec::new();

      
    for _ in 0..header.frame_count {
        match header.key_format {
            NuccAnmKeyFormat::Vector3Fixed => {
                let x = reader.read_be::<f32>()?;
                let y = reader.read_be::<f32>()?;
                let z = reader.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Vec3 { values: (x, y, z) });
            }

            NuccAnmKeyFormat::Vector3Linear => {
                let frame = reader.read_be::<i32>()?;
                let x = reader.read_be::<f32>()?;
                let y = reader.read_be::<f32>()?;
                let z = reader.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Vec3Linear { frame, values: (x, y, z) });
            }


            NuccAnmKeyFormat::QuaternionShortTable => {
                let x = reader.read_be::<i16>()?;
                let y = reader.read_be::<i16>()?;
                let z = reader.read_be::<i16>()?;
                let w = reader.read_be::<i16>()?;
                keyframes.push(NuccAnmKey::ShortVec4 { values: (x, y, z, w) });
            }
            
            NuccAnmKeyFormat::QuaternionLinear => {
                let frame = reader.read_be::<i32>()?;
                let x = reader.read_be::<f32>()?;
                let y = reader.read_be::<f32>()?;
                let z = reader.read_be::<f32>()?;
                let w = reader.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Vec4Linear { frame, values: (x, y, z, w) });
            }

            

            NuccAnmKeyFormat::FloatFixed => {
                let x = reader.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Float { values: x });
            }

            NuccAnmKeyFormat::FloatLinear => {
                let frame = reader.read_be::<i32>()?;
                let x = reader.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::FloatLinear { frame, values: x });
            }


            NuccAnmKeyFormat::OpacityShortTable => {
                let x = reader.read_be::<i16>()?;
                keyframes.push(NuccAnmKey::I16Vec { values: x });
            }

            NuccAnmKeyFormat::ScaleShortTable => {
                let x = reader.read_be::<i16>()?;
                let y = reader.read_be::<i16>()?;
                let z = reader.read_be::<i16>()?;
                keyframes.push(NuccAnmKey::I16Vec3 { values: (x, y, z) });
            }

            NuccAnmKeyFormat::ColorRGBTable => {
                let r = reader.read_be::<u8>()?;
                let g = reader.read_be::<u8>()?;
                let b = reader.read_be::<u8>()?;
                keyframes.push(NuccAnmKey::Color { values: (r, g, b) });
            }


            
            _ => todo!(),

        }
        



        
    }

    Ok(keyframes)
}


fn write_fcurve_data<R: Write + Seek>(
    anm_key: NuccAnmKeyFormat,
    values: NuccAnmKey,
    writer: &mut R,
    wo: &WriteOptions,
    
    _: ()
) -> BinResult<()> {

    match (anm_key, values) {
        (NuccAnmKeyFormat::Vector3Fixed, NuccAnmKey::Vec3 { values }) |
        (NuccAnmKeyFormat::EulerXYZFixed, NuccAnmKey::Vec3 { values }) |
        (NuccAnmKeyFormat::Vector3Table, NuccAnmKey::Vec3 { values }) => {
            values.0.write_options(writer, wo, ())?;
            values.1.write_options(writer, wo, ())?;
            values.2.write_options(writer, wo, ())?;
        }
    
        (NuccAnmKeyFormat::Vector3Linear, NuccAnmKey::Vec3Linear { frame, values }) => {
            frame.write_options(writer, wo, ())?;
            values.0.write_options(writer, wo, ())?;
            values.1.write_options(writer, wo, ())?;
            values.2.write_options(writer, wo, ())?;
        }

        (NuccAnmKeyFormat::QuaternionLinear, NuccAnmKey::Vec4Linear { frame, values }) => {
            frame.write_options(writer, wo, ())?;
            values.0.write_options(writer, wo, ())?;
            values.1.write_options(writer, wo, ())?;
            values.2.write_options(writer, wo, ())?;
            values.3.write_options(writer, wo, ())?;
        }

        (NuccAnmKeyFormat::FloatFixed, NuccAnmKey::Float { values }) |
        (NuccAnmKeyFormat::FloatTable, NuccAnmKey::Float { values}) |
        (NuccAnmKeyFormat::FloatTableNoInterp, NuccAnmKey::Float { values}) => {
            values.write_options(writer, wo, ())?;
        }

        (NuccAnmKeyFormat::FloatLinear, NuccAnmKey::FloatLinear { frame, values }) => {
            frame.write_options(writer, wo, ())?;
            values.write_options(writer, wo, ())?;
        }

        (NuccAnmKeyFormat::OpacityShortTable, NuccAnmKey::I16Vec { values }) |
        (NuccAnmKeyFormat::OpacityShortTableNoInterp, NuccAnmKey::I16Vec { values }) => {
            values.write_options(writer, wo, ())?;
        }

        (NuccAnmKeyFormat::ScaleShortTable, NuccAnmKey::I16Vec3 { values }) => {
            values.0.write_options(writer, wo, ())?;
            values.1.write_options(writer, wo, ())?;
            values.2.write_options(writer, wo, ())?;
        }

        (NuccAnmKeyFormat::QuaternionShortTable, NuccAnmKey::ShortVec4 { values }) |
        (NuccAnmKeyFormat::QuaternionShortTableNoInterp, NuccAnmKey::ShortVec4 { values }) => {
            values.0.write_options(writer, wo, ())?;
            values.1.write_options(writer, wo, ())?;
            values.2.write_options(writer, wo, ())?;
            values.3.write_options(writer, wo, ())?;
        }

        (NuccAnmKeyFormat::ColorRGBTable, NuccAnmKey::Color { values }) => {
            values.0.write_options(writer, wo, ())?;
            values.1.write_options(writer, wo, ())?;
            values.2.write_options(writer, wo, ())?;
        }
        // Handle other NuccAnmKeyFormat cases and Keyframe variants...
        _ => todo!(),
    }
    
    Ok(())
}




impl NuccChunk for NuccChunkAnm {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAnm
    }

    fn version(&self) -> u16 {
        self.version
    }

    
}
