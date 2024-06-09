use super::*;

use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct NuccCamera {

    #[pyo3(get, set)]
    pub struct_info: NuccStructInfo,

    #[pyo3(get, set)]
    pub version: u16,

    #[pyo3(get, set)]
    pub fov: f32
}


#[pymethods]
impl NuccCamera {
    #[new]
    #[pyo3(signature = (struct_info = None, version = 121, fov = 0.0))]
    pub fn __new__(
        struct_info: Option<NuccStructInfo>,
        version: u16,
        fov: f32,
    ) -> Self {
        Self {
            struct_info: struct_info.unwrap_or_default(),
            version,
            fov,
        }
    
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "NuccCamera {{ struct_info: {:?}, version: {}, fov: {} }}",
            self.struct_info, self.version, self.fov
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

impl_nucc_info!(NuccCamera, struct_info);

impl From<NuccStructConverter> for NuccCamera {
    fn from(converter: NuccStructConverter) -> Self {
        let NuccStructConverter {
            nucc_chunk,
            struct_infos: _,
            struct_references: _,
        } = converter;

        let chunk = nucc_chunk
            .downcast::<NuccChunkCamera>()
            .map(|c| *c)
            .ok()
            .unwrap();

        Self {
            struct_info: Default::default(),
            version: chunk.version,
            fov: chunk.fov,
        }
    }
}

impl From<NuccChunkConverter> for Box<NuccChunkCamera> {
    fn from(converter: NuccChunkConverter) -> Self {
        let NuccChunkConverter {
            nucc_struct,
            struct_info_map: _,
            struct_reference_map: _,
        } = converter;

        let cam = nucc_struct
            .downcast::<NuccCamera>()
            .map(|c| *c)
            .ok()
            .unwrap();

        Box::new(NuccChunkCamera {
            version: cam.version,
            fov: cam.fov,
        })
    }
}

impl NuccStruct for NuccCamera {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkCamera
    }

    fn version(&self) -> u16 {
        self.version
    }
}
