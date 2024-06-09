pub mod nucc;
pub mod nucc_chunk;
pub mod xfbin;
pub mod xfbin_file;


use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use binrw::{io::Cursor, BinWriterExt};
use std::fs::File;
use std::io::Write;

pub use xfbin::{Xfbin, XfbinPage};
pub use nucc::{NuccStructInfo, NuccStructReference};
pub use nucc::{NuccAnm, nucc_anm::Entry, NuccBinary, NuccCamera};

pub use nucc_chunk::{
    NuccChunkAnm, nucc_chunk_anm::AnmClump, nucc_chunk_anm::AnmCoord, 
    nucc_chunk_anm::CoordParent, nucc_chunk_anm::EntryFormat,
    nucc_chunk_anm::CurveHeader,
    nucc_chunk_anm::Curve, nucc_chunk_anm::CurveFormat,
    nucc_chunk_anm::Math
};

use xfbin_file::XfbinFile;


/*#[pyfunction]
pub fn read_xfbin(filepath: &str) -> PyResult<Xfbin> {
    let buffer = fs::read(filepath)?;

    let xfbin = read_xfbin_buf(buffer);

    Ok(xfbin.unwrap())


}


#[pyfunction]
pub fn read_xfbin_buf(buf: Vec<u8>) -> PyResult<Xfbin> {
    let mut reader = std::io::Cursor::new(buf);

    let xfbin_file = reader
        .read_be::<XfbinFile>().unwrap();
        

    Ok(xfbin_file.into())

    
}*/

#[pyfunction]
pub fn write_xfbin(xfbin: Xfbin, filepath: &str) -> PyResult<()> {
    let buf = write_xfbin_buf(xfbin)?;

    let mut file = File::create(filepath)?;

    Ok(file.write_all(&buf)?)
}

#[pyfunction]
pub fn write_xfbin_buf(xfbin: Xfbin) -> PyResult<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());

    let xfbin_file = XfbinFile::from(xfbin.clone());

    cursor.write_be(&xfbin_file).unwrap();

    Ok(cursor.into_inner())

    

    
}

#[pymodule]
fn xfbinlib(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    //m.add_function(wrap_pyfunction!(read_xfbin, m)?)?;
    //m.add_function(wrap_pyfunction!(read_xfbin_buf, m)?)?;
    m.add_function(wrap_pyfunction!(write_xfbin, m)?)?;
    m.add_function(wrap_pyfunction!(write_xfbin_buf, m)?)?;
    m.add_class::<Xfbin>()?;
    m.add_class::<XfbinPage>()?;
    m.add_class::<NuccAnm>()?;
    m.add_class::<NuccBinary>()?;
    m.add_class::<NuccCamera>()?;
    m.add_class::<NuccStructInfo>()?;
    m.add_class::<NuccStructReference>()?;
    m.add_class::<NuccChunkAnm>()?;
    m.add_class::<AnmClump>()?;
    m.add_class::<AnmCoord>()?;
    m.add_class::<CoordParent>()?;
    m.add_class::<EntryFormat>()?;
    m.add_class::<Entry>()?;
    m.add_class::<CurveHeader>()?;
    m.add_class::<Curve>()?;
    m.add_class::<CurveFormat>()?;
    m.add_class::<Math>()?;
    

    Ok(())
   
}