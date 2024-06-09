use super::*;


#[pyclass]
#[derive(Debug, Default, Clone)]
pub struct NuccBinary {
    #[pyo3(get, set)]
    pub struct_info: NuccStructInfo,
    #[pyo3(get, set)]
    pub version: u16,

    #[pyo3(get, set)]
    pub data: Vec<u8>,
}

#[pymethods]
impl NuccBinary {
    #[new]
    #[pyo3(signature = (struct_info = None, version = 121, data = None))]
    pub fn __new__(
        struct_info: Option<NuccStructInfo>,
        version: u16,
        data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            struct_info: struct_info.unwrap_or_default(),
            version,
            data: data.unwrap_or_default(),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "NuccBinary {{ struct_info: {:?}, version: {}, data: {:?} }}",
            self.struct_info, self.version, self.data
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

impl_nucc_info!(NuccBinary, struct_info);

impl From<NuccStructConverter> for NuccBinary {
    fn from(converter: NuccStructConverter) -> Self {
        let NuccStructConverter {
            nucc_chunk,
            struct_infos: _,
            struct_references: _,
        } = converter;

        let chunk = nucc_chunk
            .downcast::<NuccChunkBinary>()
            .map(|c| *c)
            .ok()
            .unwrap();

        Self {
            struct_info: Default::default(),
            version: chunk.version,
            data: chunk.data,
        }
    }
}

impl From<NuccChunkConverter> for Box<NuccChunkBinary> {
    fn from(converter: NuccChunkConverter) -> Self {
        let NuccChunkConverter {
            nucc_struct,
            struct_info_map: _,
            struct_reference_map: _,
        } = converter;

        let binary = nucc_struct
            .downcast::<NuccBinary>()
            .map(|c| *c)
            .ok()
            .unwrap();

        Box::new(NuccChunkBinary {
            version: binary.version,
            data: binary.data,
        })
    }
}

impl NuccStruct for NuccBinary {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkBinary
    }

    fn version(&self) -> u16 {
        self.version
    }
}
