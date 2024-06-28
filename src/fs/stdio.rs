use std::path::Path;

use serde::{Deserialize, Serialize};

use super::filesystem::FileSystem;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stdio {
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
    error_buffer: Vec<u8>,
}

impl Stdio {
    pub fn new() -> Self {
        Stdio { input_buffer: Vec::new(), output_buffer: Vec::new(), error_buffer: Vec::new() }
    
    }

    pub fn read_file<P: AsRef<Path>>(
        &mut self,
        name: &str,
        path: P,
        fs: &FileSystem,
    ) -> std::io::Result<()> {
        let data = fs.read_file(name, path)?;
        self.output_buffer = data;
        Ok(())
    }

    // pub fn write_file<P: AsRef<Path>>(
    //     &mut self,
    //     name: &str,
    //     path: P,
    //     fs: &mut FileSystem,
    // ) -> std::io::Result<()> {
    //     fs.write_file(name, &self.input_buffer, path)
    // }

    // pub fn read(&mut self) -> Vec<u8> {
    //     let data = self.input_buffer.clone();
    //     self.input_buffer.clear();
    //     data
    // }

    pub fn write(&mut self, data: &[u8]) {
        self.output_buffer.extend_from_slice(data);
    }

    pub fn error(&mut self, data: &[u8]) {
        self.error_buffer.extend_from_slice(data);
    }
    
    pub fn print(&mut self) {
        // remove the \n at the start and end of the output_buffer
        if self.output_buffer.ends_with(&[10]) {
            self.output_buffer.pop(); 
        }
        if self.output_buffer.starts_with(&[10]) {
            self.output_buffer.remove(0); 
        }
        println!("{}", String::from_utf8_lossy(&self.output_buffer));
        self.output_buffer.clear();
    }

    pub fn print_error(&mut self) {
        // remove the \n at the start and end of the error_buffer
        if self.error_buffer.ends_with(&[10]) {
            self.error_buffer.pop(); 
        }
        if self.error_buffer.starts_with(&[10]) {
            self.error_buffer.remove(0); 
        }
        eprintln!("{}", String::from_utf8_lossy(&self.error_buffer));
        self.error_buffer.clear();
    }

    // pub fn input(&mut self, data: &[u8]) {
    //   self.input_buffer.extend_from_slice(data);
    // }
}
