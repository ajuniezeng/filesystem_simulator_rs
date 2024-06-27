use std::path::Path;

use super::filesystem::FileSystem;

pub struct Stdio {
    buffer: Vec<u8>,
}

impl Stdio {
    pub fn new() -> Self {
        Stdio { buffer: Vec::new() }
    }

    pub fn read_file<P: AsRef<Path>>(
        &mut self,
        name: &str,
        path: P,
        fs: &FileSystem,
    ) -> std::io::Result<()> {
        let data = fs.read_file(name, path)?;
        self.buffer = data;
        Ok(())
    }

    pub fn write_file<P: AsRef<Path>>(
        &mut self,
        name: &str,
        path: P,
        fs: &mut FileSystem,
    ) -> std::io::Result<()> {
        fs.write_file(name, &self.buffer, path)
    }
    
    pub fn print(&self) {
      println!("{}", String::from_utf8_lossy(&self.buffer));
    }

    pub fn input(&mut self, data: &[u8]) {
      self.buffer.clear();
      self.buffer.extend_from_slice(data);
    }
}
