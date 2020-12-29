use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::io::{self, Read, Seek, Write};
use wasmer_wasi::{WasiFile, WasiFsError};

/// For piping stdio. Stores all output / input in a byte-vector.
#[derive(Debug, Serialize, Deserialize)]
pub struct Pipe {
    pub buffer: Vec<u8>,
}

impl Pipe {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.buffer[..]))
    }
}

#[typetag::serde]
impl WasiFile for Pipe {
    fn last_accessed(&self) -> u64 {
        0
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn created_time(&self) -> u64 {
        0
    }
    fn size(&self) -> u64 {
        self.buffer.len() as u64
    }
    fn set_len(&mut self, len: u64) -> Result<(), WasiFsError> {
        Ok(self.buffer.resize(len as usize, 0))
    }
    fn unlink(&mut self) -> Result<(), WasiFsError> {
        Ok(())
    }
    fn bytes_available(&self) -> Result<usize, WasiFsError> {
        Ok(self.buffer.len())
    }
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.buffer[..]).read(buf)
    }
}

impl Seek for Pipe {
    fn seek(&mut self, _pos: io::SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "can not seek in a pipe",
        ))
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
