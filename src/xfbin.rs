use binrw::NullString;
use indexmap::IndexMap;
use itertools::Itertools;

use pyo3::prelude::*;
use pyo3::types::PyList;


use super::nucc::*;
use super::nucc_chunk::*;
use super::xfbin_file::*;


#[pyclass(module = "xfbin_lib")]
#[derive(Debug, Clone)]
pub struct Xfbin {

    #[pyo3(get, set)]
    pub version: u16,

   #[pyo3(get, set)]
    pub pages: Py<PyList>
}

#[pymethods]
impl Xfbin {

    #[new]
    #[pyo3(signature = (version = None, pages = None))]
    fn __new__(py: Python, version: Option<u16>, pages: Option<Py<PyList>>) -> Self {
        Self {
            version: version.unwrap_or(121),
            pages : pages.unwrap_or(PyList::empty_bound(py).into())
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| 
        {   
           let pages: Vec<XfbinPage> = self.pages.extract(py).unwrap();
           let pages: Vec<String> = pages.iter().map(|page| page.__repr__().unwrap()).collect();

            Ok(format!(
            "Xfbin(version={}, pages={})",
            self.version, pages.join(", ")
            ))
           
        })
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct XfbinPage {
    #[pyo3(get, set)]
    pub structs: Py<PyList>,

    #[pyo3(get, set)]
    pub struct_infos: Py<PyList>,

    #[pyo3(get, set)]
    pub struct_references: Py<PyList>
}


#[pymethods]
impl XfbinPage {

    #[new]
    #[pyo3(signature = (structs = None, struct_infos = None, struct_references = None))]
    fn __new__(py: Python, structs: Option<Py<PyList>>, struct_infos: Option<Py<PyList>>, struct_references: Option<Py<PyList>>) -> Self {
        XfbinPage {
            structs: structs.unwrap_or(PyList::empty_bound(py).into()),
            struct_infos: struct_infos.unwrap_or(PyList::empty_bound(py).into()),
            struct_references: struct_references.unwrap_or(PyList::empty_bound(py).into())
        }
    }
    

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let struct_infos: Vec<NuccStructInfo> = self.struct_infos.extract(py)?;
            let struct_references: Vec<NuccStructReference> = self.struct_references.extract(py)?;
            let structs: Vec<Box<dyn NuccStruct>> = self.structs.extract(py).unwrap();

            Ok(format!(
                "XfbinPage(structs={:?}, struct_infos={:?}, struct_references={:?})",
                structs, struct_infos, struct_references
            ))
        })
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    pub fn has_unknown_chunk(&self) -> bool {
        let structs: Vec<Box<dyn NuccStruct>> = Python::with_gil(|py| {
            self.structs.extract(py).unwrap()
        });

        structs.iter().any(|nucc_struct| nucc_struct.chunk_type() == NuccChunkType::NuccChunkUnknown)
    }

    pub fn has_anm_chunk(&self) -> bool {
        let structs: Vec<Box<dyn NuccStruct>> = Python::with_gil(|py| {
            self.structs.extract(py).unwrap()
        });

        structs.iter().any(|nucc_struct| nucc_struct.chunk_type() == NuccChunkType::NuccChunkAnm)
    }


    #[allow(clippy::type_complexity)]
    pub fn destructure(&self) -> (
        Vec<Box<dyn NuccStruct>>,
        IndexMap<NuccStructInfo, u32>,
        IndexMap<NuccStructReference, u32>,
    ) {

        let structs: Vec<Box<dyn NuccStruct>> = Python::with_gil(|py| {
            self.structs.extract(py).unwrap()
        });

        let mut struct_infos = IndexMap::<NuccStructInfo, u32>::new();
        let mut struct_references = IndexMap::<NuccStructReference, u32>::new();

        let struct_infos_vec: Vec<NuccStructInfo> = Python::with_gil(|py| {
            self.struct_infos.extract(py).unwrap()
        });

        let struct_references_vec: Vec<NuccStructReference> = Python::with_gil(|py| {
            self.struct_references.extract(py).unwrap()
        });

        if self.has_unknown_chunk() || self.has_anm_chunk() {
            struct_infos.extend(struct_infos_vec.iter().enumerate().map(|(i, s)| (s.clone(), i as u32)));
            struct_references.extend(struct_references_vec.iter().enumerate().map(|(i, s)| ((*s).clone(), i as u32)));
        }

        (structs, struct_infos, struct_references)
    }

}

    
impl From<XfbinFile> for Xfbin {
    fn from(xfbin: XfbinFile) -> Self {
        let mut pages = Vec::new();

        // Create a new XfbinPage PyObj
        let mut page = Python::with_gil(
            |py| XfbinPage::__new__(py, None, None, None)
        );

        let chunk_names = xfbin
            .index
            .chunk_names
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let chunk_types = xfbin
            .index
            .chunk_types
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let filepaths = xfbin
            .index
            .filepaths
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let mut struct_infos_index: usize = 0;
        let mut struct_references_index: usize = 0;

        let struct_infos = Vec::<NuccStructInfo>::from(XfbinChunkMapConverter {
            chunk_maps: xfbin.index.chunk_maps.clone(),
            chunk_names: chunk_names.clone(),
            chunk_types: chunk_types.clone(),
            filepaths: filepaths.clone(),
        });

        let struct_references = Vec::<NuccStructReference>::from(XfbinChunkReferenceConverter {
            references: xfbin.index.chunk_references,
            chunk_names,
            struct_infos: struct_infos.clone(),
        });

        let struct_infos_mapped = xfbin
            .index
            .chunk_map_indices
            .iter()
            .map(|&i| struct_infos[i as usize].clone())
            .collect::<Vec<NuccStructInfo>>();

        for chunk in xfbin.chunks {
            let NuccStructInfo {
                chunk_name,
                chunk_type,
                filepath,
            } = &struct_infos_mapped[struct_infos_index + chunk.chunk_map_index as usize].clone();

            let parsed = chunk.unpack(chunk_type);

            match parsed.chunk_type() {
                NuccChunkType::NuccChunkNull => continue,
                NuccChunkType::NuccChunkPage => {
                    let NuccChunkPage {
                        version: _,
                        map_index_count: struct_infos_count,
                        reference_count: struct_references_count,
                    } = parsed.downcast::<NuccChunkPage>().map(|c| *c).ok().unwrap();

                    let struct_infos_count = struct_infos_count as usize;
                    let struct_references_count = struct_references_count as usize;

                    // Convert Vec<NuccStructInfo> to Py<PyList>
                    let struct_infos = Python::with_gil(|py| {
                        PyList::new_bound(py, struct_infos_mapped[struct_infos_index..(struct_infos_index + struct_infos_count)]
                            .iter()
                            .map(|struct_info| Py::new(py, struct_info.clone()).unwrap())
                        ).into()
                    });

                    page.struct_infos = struct_infos;

                    // Convert Vec<NuccStructReference> to Py<PyList>
                    let struct_references: Py<PyList> = Python::with_gil(|py| {
                        PyList::new_bound(py, struct_references[struct_references_index
                            ..(struct_references_index + struct_references_count)]
                            .iter()
                            .map(|struct_reference| Py::new(py, struct_reference.clone()).unwrap())
                        ).into()
                    });

                    page.struct_references = struct_references;

                    pages.push(page);
                    
                    page = Python::with_gil(
                        |py| XfbinPage::__new__(py, None, None, None)
                    );

                    struct_infos_index += struct_infos_count;
                    struct_references_index += struct_references_count;

                    continue;
                }
                _ => (),
            }

            let mut parsed_struct = Box::<dyn NuccStruct>::from(NuccStructConverter {
                nucc_chunk: parsed,
                struct_infos: struct_infos_mapped.clone(),
                struct_references: struct_references.clone(),
            });
            
            
        
            let struct_info = parsed_struct.struct_info_mut();
            struct_info.chunk_name = chunk_name.clone();
            struct_info.filepath = filepath.clone();
            struct_info.chunk_type = chunk_type.clone();


            let structs: Py<PyList> = Python::with_gil(|py| {
                let mut structs: Vec<PyObject> = page.structs.extract(py).unwrap();

                let py_struct: Py<PyAny> = Python::with_gil(|py| {
                    let py_struct = parsed_struct.into_py(py);
                    
                    py_struct 
                });
                
                structs.push(py_struct.into());
                PyList::new_bound(py, structs).into()
            });


            page.structs = structs;

        }

        let pages = Python::with_gil(|py| {
            PyList::new_bound(py, pages.iter().map(|page| {
                Py::new(py, page.clone()).unwrap()
            })).into()
        });

        Self {
            version: xfbin.header.version as u16,
            pages,
        }
    }
}

fn repack_struct(
    boxed: Box<dyn NuccChunk>,
    struct_info: NuccStructInfo,
    page_struct_infos: &mut IndexMap<NuccStructInfo, u32>,
) -> XfbinChunk {
    let struct_info_index = page_struct_infos.len() as u32;

    let chunk_map_index = *page_struct_infos
        .entry(struct_info)
        .or_insert(struct_info_index);

    let mut chunk = XfbinChunk::repack(boxed);
    chunk.chunk_map_index = chunk_map_index;

    chunk
}

impl From<Xfbin> for XfbinFile {
    fn from(xfbin: Xfbin) -> Self {
        let mut header = XfbinHeader::default();
        header.version = 121 as u32;

        let mut index = XfbinIndex::default();
        index.version = 121 as u16;

        let mut min_page_size = 0;

        let mut chunks = vec![];

        let mut struct_infos_map = IndexMap::<NuccStructInfo, u32>::new();

        let mut chunk_map_indices = vec![];
        let mut struct_references_vec = vec![];

        let null_chunk = repack_struct(
            Box::new(NuccChunkNull(xfbin.version)),
            NuccChunkNull::default_chunk_info(),
            &mut struct_infos_map,
        );

        chunks.push(null_chunk);


        // Convert Py<PyList> to Vec<XfbinPage>
        let pages: Vec<XfbinPage> = Python::with_gil(|py| {
            xfbin.pages.extract(py).unwrap()
        });

        for page in pages {
            let (page_structs, mut page_struct_infos, page_struct_references) = page.destructure();

            let null_chunk = repack_struct(
                Box::new(NuccChunkNull(xfbin.version)),
                NuccChunkNull::default_chunk_info(),
                &mut page_struct_infos,
            );

            chunks.push(null_chunk);

            for nucc_struct in page_structs {
                let struct_info = nucc_struct.struct_info().clone();

                let boxed = Box::<dyn NuccChunk>::from(NuccChunkConverter {
                    nucc_struct,
                    struct_info_map: page_struct_infos.clone(),
                    struct_reference_map: page_struct_references.clone(),
                });

                chunks.push(repack_struct(boxed, struct_info, &mut page_struct_infos));
            }

            // Add nuccChunkPage map
            repack_struct(
                Box::new(NuccChunkPage::default()),
                NuccChunkPage::default_chunk_info(),
                &mut page_struct_infos,
            );

            // Add nuccChunkIndex map
            repack_struct(
                Box::new(NuccChunkIndex),
                NuccChunkIndex::default_chunk_info(),
                &mut page_struct_infos,
            );

         

          
            // Create final nuccChunkPage
            let page_chunk = repack_struct(
                Box::new(NuccChunkPage {
                    version: xfbin.version,
                    map_index_count: page_struct_infos.len() as u32,
                    reference_count: page_struct_references.len() as u32,
                }),
                NuccChunkPage::default_chunk_info(),
                &mut page_struct_infos,
            );

            chunks.push(page_chunk);


            for struct_info in page_struct_infos
            .clone()
            .into_iter()
            .sorted_by_key(|(_, v)| *v)
            .map(|(k, _)| k)
        {
            let struct_info_index = struct_infos_map.len() as u32;
            chunk_map_indices.push(
                *struct_infos_map
                    .entry(struct_info)
                    .or_insert(struct_info_index),
            );

        }

            struct_references_vec.extend(
                page_struct_references
                    .into_iter()
                    .sorted_by_key(|(_, v)| *v)
                    .map(|(k, _)| k),
            );

            // Get the smallest ie the minimum page_struct_infos length for structs that are not Null or Page
            if min_page_size == 0 || page_struct_infos.len() > min_page_size {
                min_page_size = page_struct_infos.len() - 3;
            }
            

        }

        let mut chunk_type_map = IndexMap::new();
        let mut file_path_map = IndexMap::new();
        let mut chunk_name_map = IndexMap::new();


        let chunk_maps = struct_infos_map
            .clone()
            .into_iter()
            .sorted_by_key(|(_, v)| *v)
            .map(|(struct_info, _)| {
                let mut chunk_type_index = chunk_type_map.len() as u32;
                chunk_type_index = *chunk_type_map
                    .entry(struct_info.chunk_type.clone())
                    .or_insert(chunk_type_index);

                let mut filepath_index = file_path_map.len() as u32;
                filepath_index = *file_path_map
                    .entry(struct_info.filepath.clone())
                    .or_insert(filepath_index);

                let mut chunk_name_index = chunk_name_map.len() as u32;
                chunk_name_index = *chunk_name_map
                    .entry(struct_info.chunk_name.clone())
                    .or_insert(chunk_name_index);

                XfbinChunkMap {
                    chunk_type_index,
                    filepath_index,
                    chunk_name_index,
                }
            })
            .collect::<Vec<XfbinChunkMap>>();

        let chunk_references = struct_references_vec
            .iter()
            .map(|struct_reference| {
                let mut chunk_name_index = chunk_name_map.len() as u32;
                chunk_name_index = *chunk_name_map
                    .entry(struct_reference.chunk_name.clone())
                    .or_insert(chunk_name_index);

                let mut chunk_map_index = struct_infos_map.len() as u32;
                chunk_map_index = *struct_infos_map
                    .entry(struct_reference.struct_info.clone())
                    .or_insert(chunk_map_index);

                XfbinChunkReference {
                    chunk_name_index,
                    chunk_map_index,
                }
            })
            .collect::<Vec<XfbinChunkReference>>();

        let chunk_types = chunk_type_map
            .into_iter()
            .sorted_by_key(|(_, v)| *v)
            .map(|(k, _)| NullString::from(k))
            .collect_vec();

        let filepaths = file_path_map
            .into_iter()
            .sorted_by_key(|(_, v)| *v)
            .map(|(k, _)| NullString::from(k))
            .collect_vec();

        let chunk_names = chunk_name_map
            .into_iter()
            .sorted_by_key(|(_, v)| *v)
            .map(|(k, _)| NullString::from(k))
            .collect_vec();
        

        index.min_page_size = min_page_size as u32;
        index.chunk_types = chunk_types;
        index.filepaths = filepaths;
        index.chunk_names = chunk_names;

        index.chunk_maps = chunk_maps;

        index.chunk_references = chunk_references;
        index.chunk_map_indices = chunk_map_indices;

        let xfbin_file = Self {
            header,
            index,
            chunks,
        };

        xfbin_file
    }
}