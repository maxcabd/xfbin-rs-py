use super::*;

use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::nucc_chunk::nucc_chunk_anm::{AnmClump, CoordParent, AnmCoord, AnmEntry, TrackHeader, AnmTrack, NuccAnmKey, EntryFormat};


#[derive(Debug, Clone)]
#[pyclass]
pub struct NuccAnm {

    #[pyo3(get, set)]
    pub struct_info: NuccStructInfo,

    #[pyo3(get, set)]
    pub version: u16,

    #[pyo3(get, set)]
    pub frame_count: u32,

    #[pyo3(get, set)]
    pub is_looped: bool,

    #[pyo3(get, set)]
    pub other_entries_indices: Vec<u32>,

    #[pyo3(get, set)]
    pub unk_entry_indices: Vec<u32>,

    #[pyo3(get, set)]
    pub clumps: Py<PyList>,

    #[pyo3(get, set)]
    pub coord_parents: Py<PyList>,

    #[pyo3(get, set)]
    pub entries: Py<PyList>,
}

#[derive(Debug, Clone)]
#[pyclass(name = "AnmEntry")]
pub struct Entry {
    #[pyo3(get, set)]
    pub coord: AnmCoord,

    #[pyo3(get, set)]
    pub entry_format: EntryFormat,

    #[pyo3(get, set)]
    pub track_headers: Py<PyList>,

    #[pyo3(get, set)]
    pub tracks: Py<PyList>,
}

#[pymethods]
impl Entry {
    #[new]
    #[pyo3(signature = (coord = None, entry_format = None, track_headers = None, tracks = None))]
    pub fn __new__(
        py: Python,
        coord: Option<AnmCoord>,
        entry_format: Option<EntryFormat>,
        track_headers: Option<Py<PyList>>,
        tracks: Option<Py<PyList>>,
    ) -> Self {
        Self {
            coord: coord.unwrap_or_default(),
            entry_format: entry_format.unwrap_or_default(),
            track_headers: track_headers.unwrap_or(PyList::empty_bound(py).into()),
            tracks: tracks.unwrap_or(PyList::empty_bound(py).into()),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let track_headers: Vec<TrackHeader> = self.track_headers.extract(py)?;
            let tracks: Vec<Track> = self.tracks.extract(py)?;

            // Use original __repr__ methods 
            let tracks: Vec<String> = tracks.iter().map(|track| track.__repr__().unwrap()).collect();
            let track_headers: Vec<String> = track_headers.iter().map(|curve_header| curve_header.__repr__().unwrap()).collect();
    
        Ok(format!(
            "AnmEntry(coord={:?}, entry_format=EntryFormat.{:?}, track_headers={}, tracks={})",
            self.coord, self.entry_format, track_headers.join(", "), tracks.join(", ")
        ))
        })
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct Track {
    #[pyo3(get, set)]
    pub keys: Py<PyList>,
}

#[pymethods]
impl Track {
    #[new]
    #[pyo3(signature = (keys = None))]
    pub fn __new__(
        py: Python,
        keys: Option<Py<PyList>>,
    ) -> Self {
        Self {
            keys: keys.unwrap_or(PyList::empty_bound(py).into()),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let keys: Vec<NuccAnmKey> = self.keys.extract(py)?;

        Ok(format!(
            "NuccAnmKey(keyframes={:?})", keys))
        })
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

}

#[pymethods]
impl NuccAnm {
    #[new]
    #[pyo3(signature = (struct_info = None, version = 121, frame_count = 0, is_looped = false, other_entries_indices = None, unk_entry_indices = None, clumps = None, coord_parents = None, entries = None))]
    pub fn __new__(
        py: Python,
        struct_info: Option<NuccStructInfo>,
        version: u16,
        frame_count: u32,
        is_looped: bool,
        other_entries_indices: Option<Vec<u32>>,
        unk_entry_indices: Option<Vec<u32>>,
        clumps: Option<Py<PyList>>,
        coord_parents: Option<Py<PyList>>,
        entries: Option<Py<PyList>>,
    ) -> Self {
        Self {
            struct_info: struct_info.unwrap_or_default(),
            version,
            frame_count,
            is_looped,
            other_entries_indices: other_entries_indices.unwrap_or_default(),
            unk_entry_indices: unk_entry_indices.unwrap_or_default(),
            clumps: clumps.unwrap_or(PyList::empty_bound(py).into()),
            coord_parents: coord_parents.unwrap_or(PyList::empty_bound(py).into()),
            entries: entries.unwrap_or(PyList::empty_bound(py).into()),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "NuccAnm(struct_info={:?}, version={}, frame_count={}, is_looped={}, other_entries_indices={:?}, unk_entry_indices={:?} clumps={:?}, coord_parents={:?}, entries={:?})",
            self.struct_info, self.version, self.frame_count, self.is_looped, self.other_entries_indices, self.unk_entry_indices, self.clumps,  self.coord_parents, self.entries
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

}

impl_nucc_info!(NuccAnm, struct_info);

impl From<NuccStructConverter> for NuccAnm {
    fn from(converter: NuccStructConverter) -> Self {
        let NuccStructConverter {
            nucc_chunk,
            struct_infos: _,
            struct_references: _,
        } = converter;

        let chunk = nucc_chunk
            .downcast::<NuccChunkAnm>()
            .map(|c| *c)
            .ok()
            .unwrap();
        

        let clumps: Py<PyList> = Python::with_gil(|py| {
            let clumps: Vec<PyObject> = chunk.clumps.iter().map(|clump| {
                let py_clump = Py::new(py, clump.clone()).unwrap();
                py_clump.into_py(py)
            }).collect();

            PyList::new_bound(py, clumps).into()
        });

        let coord_parents: Py<PyList> = Python::with_gil(|py| {
            let coord_parents: Vec<PyObject> = chunk.coord_parents.iter().map(|coord_parent| {
                let py_coord_parent = Py::new(py, coord_parent.clone()).unwrap();
                py_coord_parent.into_py(py)
            }).collect();

            PyList::new_bound(py, coord_parents).into()
        });


        let entries = Python::with_gil(|py| {
            let entries: Vec<PyObject> = chunk.entries.iter().map(|entry| {
                let track_headers: Py<PyList> = Python::with_gil(|py| {
                    let track_headers: Vec<PyObject> = entry.track_headers.iter().map(|curve_header| {
                        let py_curve_header = Py::new(py, curve_header.clone()).unwrap();
                        py_curve_header.into_py(py)
                    }).collect();

                    PyList::new_bound(py, track_headers).into()
                });

                let tracks: Py<PyList> = Python::with_gil(|py| {
                    let tracks: Vec<PyObject> = entry.tracks.iter().map(|anm_track| {
                        let keys: Py<PyList> = Python::with_gil(|py| {
                            let keys: Vec<PyObject> = anm_track.keys.iter().map(|value| {
                                let key = Py::new(py, value.clone()).unwrap();
                                key.into_py(py)
                            }).collect();

                            PyList::new_bound(py, keys).into()
                        });

                        let track = Track {
                            keys,
                        };

                        let py_channel = Py::new(py, track).unwrap();
                        py_channel.into_py(py)
                    }).collect();

                    PyList::new_bound(py, tracks).into()
                });

                let entry = Entry {
                    coord: entry.coord.clone(),
                    entry_format: entry.entry_format.clone(),
                    track_headers,
                    tracks,
                };

                let py_entry = Py::new(py, entry).unwrap();
                py_entry.into_py(py)
            }).collect();

            PyList::new_bound(py, entries).into()
        });


        Self {
            struct_info: Default::default(),
            version: chunk.version,
            frame_count: chunk.frame_count,
            is_looped: chunk.is_looped == 1,
            clumps,
            other_entries_indices: chunk.other_entries_indices,
            unk_entry_indices: chunk.unk_entry_indices,
            coord_parents,
            entries,
        }
    }
}

impl From<NuccChunkConverter> for Box<NuccChunkAnm> {
    fn from(converter: NuccChunkConverter) -> Self {
        let NuccChunkConverter {
            nucc_struct,
            struct_info_map: _,
            struct_reference_map: _,
        } = converter;

        let anm = nucc_struct.downcast::<NuccAnm>().map(|s| *s).ok().unwrap();

        let clumps: Vec<AnmClump> = Python::with_gil(|py| {
            anm.clumps.extract(py).unwrap()
        });

        let coord_parents: Vec<CoordParent> = Python::with_gil(|py| {
            anm.coord_parents.extract(py).unwrap()
        });

        let entries: Vec<Entry> = Python::with_gil(|py| {
            anm.entries.extract(py).unwrap()
        });


        let anm_entries: Vec<AnmEntry> = entries.iter().map(|entry| {
            let track_headers: Vec<TrackHeader> = Python::with_gil(|py| {
                entry.track_headers.extract(py).unwrap()
            });

            let anm_tracks: Vec<AnmTrack> = Python::with_gil(|py| {
                let tracks: Vec<Track> = entry.tracks.extract(py).unwrap();
                tracks.iter().map(|track| {
                    let keys: Vec<NuccAnmKey> = track.keys.extract(py).unwrap();
                    AnmTrack {
                        keys
                    }
                }).collect()
            });

            AnmEntry {
                coord: entry.coord.clone(),
                entry_format: entry.entry_format.clone(),
                track_headers,
                tracks: anm_tracks,
            }
        }).collect();

        let mut chunk = NuccChunkAnm::default();
        chunk.version = anm.version;
        chunk.frame_count = anm.frame_count;
        chunk.is_looped = if anm.is_looped { 1 } else { 0 };
        chunk.clumps = clumps;
        chunk.other_entries_indices = anm.other_entries_indices;
        chunk.unk_entry_indices = anm.unk_entry_indices;
        chunk.coord_parents = coord_parents;
        chunk.entries = anm_entries;

        Box::new(chunk)
    }
}

impl NuccStruct for NuccAnm {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAnm
    }

    fn version(&self) -> u16 {
        self.version
    }
}


