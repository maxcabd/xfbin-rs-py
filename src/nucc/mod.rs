pub mod nucc_binary;
pub mod nucc_anm;
pub mod nucc_anmstrm;
pub mod nucc_anmstrmframe;
pub mod nucc_camera;
pub mod nucc_lightdirc;
pub mod nucc_lightpoint;
pub mod nucc_layerset;
pub mod nucc_ambient;
pub mod nucc_morphmodel;

pub mod nucc_unknown;

use pyo3::prelude::*;
use downcast_rs::{impl_downcast, Downcast};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

use super::nucc_chunk::*;
use super::xfbin_file::{XfbinChunkMap, XfbinChunkReference};

pub use nucc_binary::NuccBinary;
pub use nucc_anm::NuccAnm;
pub use nucc_anmstrm::NuccAnmStrm;
pub use nucc_anmstrmframe::NuccAnmStrmFrame;
pub use nucc_camera::NuccCamera;
pub use nucc_lightdirc::NuccLightDirc;
pub use nucc_lightpoint::NuccLightPoint;
pub use nucc_layerset::NuccLayerSet;
pub use nucc_ambient::NuccAmbient;
pub use nucc_morphmodel::NuccMorphModel;
pub use nucc_unknown::NuccUnknown;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash)]
pub struct NuccStructInfo {

    #[pyo3(get, set)]
    pub chunk_name: String,

    #[pyo3(get, set)]
    pub chunk_type: String,

    #[pyo3(get, set)]
    pub filepath: String,
}

#[pymethods]
impl NuccStructInfo {
    #[new]
    #[pyo3(signature = (chunk_name = None, chunk_type = None, filepath = None))]
    fn __new__(chunk_name: Option<String>, chunk_type: Option<String>, filepath: Option<String>) -> Self {
        Self {
            chunk_name: chunk_name.unwrap_or_default(),
            chunk_type: chunk_type.unwrap_or_default(),
            filepath: filepath.unwrap_or_default(),
        }
    }
    
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "NuccStructInfo(Name=\"{}\", Type=\"{}\", Path=\"{}\")",
            self.chunk_name, self.chunk_type, self.filepath
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    
}

impl fmt::Display for NuccStructInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ Name: \"{}\", Type: \"{}\", Path: \"{}\" }}",
            self.chunk_name, self.chunk_type, self.filepath
        )
    }
}

pub struct XfbinChunkMapConverter {
    pub chunk_maps: Vec<XfbinChunkMap>,
    pub chunk_names: Vec<String>,
    pub chunk_types: Vec<String>,
    pub filepaths: Vec<String>,
}

impl From<XfbinChunkMapConverter> for Vec<NuccStructInfo> {
    fn from(converter: XfbinChunkMapConverter) -> Self {
        let XfbinChunkMapConverter {
            chunk_maps,
            chunk_names: names,
            chunk_types: types,
            filepaths: paths,
        } = converter;

        chunk_maps
            .into_iter()
            .map(|c| NuccStructInfo {
                chunk_name: names[c.chunk_name_index as usize].clone(),
                chunk_type: types[c.chunk_type_index as usize].clone(),
                filepath: paths[c.filepath_index as usize].clone(),
            })
            .collect()
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash)]
pub struct NuccStructReference {

    #[pyo3(get, set)]
    pub chunk_name: String,

    #[pyo3(get, set)]
    pub struct_info: NuccStructInfo,
}

#[pymethods]
impl NuccStructReference {
    #[new]
    #[pyo3(signature = (chunk_name = None, struct_info = None))]
    fn __new__(chunk_name: Option<String>, struct_info: Option<NuccStructInfo>) -> Self {
        Self {
            chunk_name: chunk_name.unwrap_or_default(),
            struct_info: struct_info.unwrap_or_default(),
        }
    }
    

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("NuccStructReference(Name=\"{}\", struct_info={})", self.chunk_name, self.struct_info))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    
    fn __eq__(&self, other: &Self) -> PyResult<bool> {
        Ok(self.chunk_name == other.chunk_name && self.struct_info == other.struct_info)
    }

    fn __hash__(&self) -> PyResult<usize> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.chunk_name.hash(&mut hasher);
        Ok(hasher.finish() as usize)
    }
}

impl fmt::Display for NuccStructReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Name: \"{}\", Info: {} }}", self.chunk_name, self.struct_info)
    }
}


pub struct XfbinChunkReferenceConverter {
    pub references: Vec<XfbinChunkReference>,
    pub chunk_names: Vec<String>,
    pub struct_infos: Vec<NuccStructInfo>,
}

impl From<XfbinChunkReferenceConverter> for Vec<NuccStructReference> {
    fn from(converter: XfbinChunkReferenceConverter) -> Self {
        let XfbinChunkReferenceConverter {
            references,
            chunk_names: names,
            struct_infos: infos,
        } = converter;

        references
            .into_iter()
            .map(|r| NuccStructReference {
                chunk_name: names[r.chunk_name_index as usize].clone(),
                struct_info: infos[r.chunk_map_index as usize].clone(),
            })
            .collect()
    }
}

pub trait NuccInfo {
    fn struct_info(&self) -> &NuccStructInfo;
    fn struct_info_mut(&mut self) -> &mut NuccStructInfo;
}

macro_rules! impl_nucc_info {
    ($struct:ident,$field:ident) => {
        impl NuccInfo for $struct {
            fn struct_info(&self) -> &NuccStructInfo {
                &self.$field
            }

            fn struct_info_mut(&mut self) -> &mut NuccStructInfo {
                &mut self.$field
            }
        }
    };
}

pub(crate) use impl_nucc_info;

pub trait NuccStruct: NuccInfo + Downcast + Send {
    fn chunk_type(&self) -> NuccChunkType;
    fn version(&self) -> u16;
}

impl std::fmt::Debug for dyn NuccStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NuccStruct(chunk_type={:?}, version={})",
            self.chunk_type(),
            self.version()
        )
    }
}

impl Clone for Box<dyn NuccStruct> {
    fn clone(&self) -> Self {
        // Match on the concrete types implementing `NuccStruct` and clone them before boxing
        match self.chunk_type() {
            
            NuccChunkType::NuccChunkAnm => {
                let nucc_anm: &NuccAnm = self.downcast_ref().unwrap();
                Box::new(nucc_anm.clone()) as Box<dyn NuccStruct>
            }

            NuccChunkType::NuccChunkBinary => {
                let nucc_binary: &NuccBinary = self.downcast_ref().unwrap();
                Box::new(nucc_binary.clone()) as Box<dyn NuccStruct>
            }

            NuccChunkType::NuccChunkCamera => {
                let nucc_camera: &NuccCamera = self.downcast_ref().unwrap();
                Box::new(nucc_camera.clone()) as Box<dyn NuccStruct>
            }
            // Add other cases for the remaining concrete types
            _ => panic!("Unsupported NuccStruct type for cloning"),
        }
    }
}

impl IntoPy<PyObject> for Box<dyn NuccStruct> {
    fn into_py(self, py: Python) -> PyObject {
        // Match on the concrete types implementing `NuccStruct` and convert them to Python objects
        match self.chunk_type() {

            NuccChunkType::NuccChunkAnm => {
                let nucc_anm: Box<NuccAnm> = self.downcast().unwrap();
                nucc_anm.into_py(py)
            }

            NuccChunkType::NuccChunkBinary => {
                let nucc_binary: Box<NuccBinary> = self.downcast().unwrap();
                nucc_binary.into_py(py)
            }

            NuccChunkType::NuccChunkCamera => {
                let nucc_camera: Box<NuccCamera> = self.downcast().unwrap();
                nucc_camera.into_py(py)
            }

            NuccChunkType::NuccChunkUnknown => {
                let nucc_unknown: Box<NuccUnknown> = self.downcast().unwrap();
                nucc_unknown.into_py(py)
            }
            // Add other cases for the remaining concrete types
            _ => panic!("Unsupported NuccStruct type for conversion to PyAny"),
        }
    }
}


impl<'a> FromPyObject<'a> for Box<dyn NuccStruct> {
    fn extract(obj: &'a PyAny) -> PyResult<Self> {        
        if let Ok(nucc_anm) = obj.extract::<NuccAnm>() {
            return Ok(Box::new(nucc_anm));
        }

    
        if let Ok(nucc_binary) = obj.extract::<NuccBinary>() {
            return Ok(Box::new(nucc_binary));
        }

        if let Ok(nucc_camera) = obj.extract::<NuccCamera>() {
            return Ok(Box::new(nucc_camera));
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Unsupported NuccStruct type for conversion from PyAny",
        ))
    }
}


impl_downcast!(NuccStruct);


pub struct NuccStructConverter {
    pub nucc_chunk: Box<dyn NuccChunk>,
    pub struct_infos: Vec<NuccStructInfo>,
    pub struct_references: Vec<NuccStructReference>,
}

pub struct NuccChunkConverter {
    pub nucc_struct: Box<dyn NuccStruct>,
    pub struct_info_map: IndexMap<NuccStructInfo, u32>,
    pub struct_reference_map: IndexMap<NuccStructReference, u32>,
}

impl From<NuccStructConverter> for Box<dyn NuccStruct> {
    fn from(converter: NuccStructConverter) -> Self {
        match converter.nucc_chunk.chunk_type() {
            NuccChunkType::NuccChunkBinary => Box::new(NuccBinary::from(converter)),
            NuccChunkType::NuccChunkAnm => Box::new(NuccAnm::from(converter)),
            NuccChunkType::NuccChunkAnmStrm => Box::new(NuccAnmStrm::from(converter)),
            NuccChunkType::NuccChunkAnmStrmFrame => Box::new(NuccAnmStrmFrame::from(converter)),
            NuccChunkType::NuccChunkCamera => Box::new(NuccCamera::from(converter)),
            NuccChunkType::NuccChunkLightDirc => Box::new(NuccLightDirc::from(converter)),
            NuccChunkType::NuccChunkLightPoint => Box::new(NuccLightPoint::from(converter)),
            NuccChunkType::NuccChunkLayerSet => Box::new(NuccLayerSet::from(converter)),
            NuccChunkType::NuccChunkAmbient => Box::new(NuccAmbient::from(converter)),
            NuccChunkType::NuccChunkMorphModel => Box::new(NuccMorphModel::from(converter)),
            NuccChunkType::NuccChunkUnknown => Box::new(NuccUnknown::from(converter)),
            any => panic!("Unexpected NuccChunkType: {any}"),
        }
    }
}

impl From<NuccChunkConverter> for Box<dyn NuccChunk> {
    fn from(converter: NuccChunkConverter) -> Self {
        match converter.nucc_struct.chunk_type() {
            NuccChunkType::NuccChunkBinary => { Box::<NuccChunkBinary>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkAnm => { Box::<NuccChunkAnm>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkAnmStrm => { Box::<NuccChunkAnmStrm>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkAnmStrmFrame => { Box::<NuccChunkAnmStrmFrame>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkCamera => { Box::<NuccChunkCamera>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkLightDirc => { Box::<NuccChunkLightDirc>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkLightPoint => { Box::<NuccChunkLightPoint>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkLayerSet => { Box::<NuccChunkLayerSet>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkAmbient => { Box::<NuccChunkAmbient>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkMorphModel => { Box::<NuccChunkMorphModel>::from(converter) as Box<dyn NuccChunk> }
            NuccChunkType::NuccChunkUnknown => { Box::<NuccChunkUnknown>::from(converter) as Box<dyn NuccChunk> }



            any => panic!("Unexpected NuccChunkType: {any}"),
        }
    }
}