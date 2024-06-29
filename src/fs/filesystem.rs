use std::{
    collections::{HashMap, VecDeque},
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::user::User;

use super::stdio::Stdio;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum FilePermission {
    Readable,
    Writable,
    Executable,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    name: String,
    file_type: FileType,
    permission: Option<Vec<FilePermission>>,
    owned_user: Option<String>,
    size: u64,
    pages: Vec<u64>,
    parent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystem {
    files: HashMap<String, FileEntry>,
    next_offset: u64,
    pub current_path: String,
    current_user: User,
    free_list: VecDeque<u64>,
    stdio: Stdio,
}

impl FileSystem {
    pub fn new(user: User) -> Self {
        let mut fs = FileSystem {
            files: HashMap::new(),
            next_offset: 0,
            current_path: "/".to_string(),
            current_user: user.clone(),
            free_list: VecDeque::new(),
            stdio: Stdio::new(),
        };

        // Create the root directory
        fs.files.insert(
            "/".to_string(),
            FileEntry {
                name: "/".to_string(),
                file_type: FileType::Directory,
                permission: Some([FilePermission::Readable, FilePermission::Writable].to_vec()),
                owned_user: Some(user.get_user_name()),
                size: 0,
                pages: Vec::new(),
                parent: None,
            },
        );
        fs
    }

    pub fn get_full_path(&self, name: &str) -> String {
        if name.starts_with("/") {
            name.to_string()
        } else {
            if self.current_path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", self.current_path, name)
            }
        }
    }

    fn extract_parent_paths(path: &str) -> String {
        let path = Path::new(path);
        match path.parent() {
            Some(parent) => parent.to_str().unwrap_or("").to_string(),
            None => "".to_string(),
        }
    }

    pub fn create_file(
        &mut self,
        name: &str,
        file_type: FileType,
        user: &str,
        permission: Option<Vec<FilePermission>>,
    ) -> std::io::Result<()> {
        let full_path = self.get_full_path(name);

        if self.files.contains_key(&full_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File or Directory already exists",
            ));
        }

        let parent_path = if name.starts_with("/") {
            Some(FileSystem::extract_parent_paths(name))
        } else {
            if self.current_path == "/" {
                Some(format!("/{}", FileSystem::extract_parent_paths(name)))
            } else if name.contains("/") {
                Some(format!(
                    "{}/{}",
                    self.current_path,
                    FileSystem::extract_parent_paths(name)
                ))
            } else {
                Some(self.current_path.clone())
            }
        };

        let entry = FileEntry {
            name: full_path.clone(),
            file_type,
            permission: permission.clone(),
            owned_user: Some(user.to_string()),
            size: 0,
            pages: Vec::new(),
            parent: parent_path.clone(),
        };

        self.files.insert(full_path.clone(), entry);

        // if let FileType::Directory = file_type {
        //     self.files.insert(
        //         format!("{}/.", full_path.clone()),
        //         FileEntry {
        //             name: ".".to_string(),
        //             file_type: FileType::Directory,
        //             permission: permission.clone(),
        //             owned_user: Some(user.to_string()),
        //             size: 0,
        //             pages: Vec::new(),
        //             parent: Some(full_path.clone()),
        //         },
        //     );

        //     self.files.insert(
        //         format!("{}/..", full_path),
        //         FileEntry {
        //             name: "..".to_string(),
        //             file_type: FileType::Directory,
        //             permission,
        //             owned_user: Some(user.to_string()),
        //             size: 0,
        //             pages: Vec::new(),
        //             parent: parent_path,
        //         },
        //     );
        // }

        Ok(())
    }

    pub fn allocate_page(&mut self, size: u64, path: &str) -> std::io::Result<u64> {
        let offset = if let Some(free_offset) = self.free_list.pop_front() {
            free_offset
        } else {
            let new_offset = self.next_offset;
            self.next_offset += size;
            new_offset
        };

        let mut file = OpenOptions::new().write(true).open(path)?;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&vec![0; size as usize])?;
        Ok(offset)
    }

    pub fn write_file<P: AsRef<Path>>(
        &mut self,
        name: &str,
        data: &[u8],
        path: P,
    ) -> std::io::Result<()> {
        let full_path = self.get_full_path(name);
        let page_size = 1024;

        let entry = self.files.get(&full_path);
        let entry_owned_user = entry.unwrap().owned_user.clone();

        if self.current_user.get_user_name() != "root" {
            // Check if the entry_owned_user is not None and unwrap safely
            if let Some(owned_user) = entry_owned_user {
                if owned_user != self.current_user.get_user_name() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "Permission denied",
                    ));
                }
            } else {
                // Handle the case where no user is set as the owner, if necessary
                // This could be an error or a warning depending on your design
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "No owner set for this file/directory",
                ));
            }
        }

        if let Some(entry) = self.files.get(&full_path) {
            let mut pages = entry.pages.clone();
            let mut size = entry.size;

            let mut file = OpenOptions::new().write(true).open(&path)?;
            let mut data_written = 0;

            while data_written < data.len() {
                if size % page_size == 0 {
                    let new_page_offset =
                        self.allocate_page(page_size as u64, path.as_ref().to_str().unwrap())?;
                    pages.push(new_page_offset);
                }

                let current_page = pages.last().unwrap();
                let offset_in_page = size % page_size;
                let write_size = std::cmp::min(
                    page_size - offset_in_page,
                    (data.len() - data_written) as u64,
                );

                file.seek(SeekFrom::Start(*current_page + offset_in_page as u64))?;
                file.write_all(&data[data_written..data_written + write_size as usize])?;

                size += write_size;
                data_written += write_size as usize;
            }

            if let Some(entry) = self.files.get_mut(&full_path) {
                entry.pages = pages;
                entry.size = size;
            }

            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))
        }
    }

    pub fn read_file<P: AsRef<Path>>(&self, name: &str, path: P) -> std::io::Result<Vec<u8>> {
        let full_path = self.get_full_path(name);
        if let Some(entry) = self.files.get(&full_path) {
            let page_size = 1024;
            let mut file = OpenOptions::new().read(true).open(path)?;
            let mut buffer = Vec::with_capacity(entry.size as usize);

            for (i, page_offset) in entry.pages.iter().enumerate() {
                let mut page_buffer = vec![0; page_size];
                file.seek(SeekFrom::Start(*page_offset))?;
                let bytes_to_read = if i == entry.pages.len() - 1 {
                    (entry.size % page_size as u64) as usize
                } else {
                    page_size
                };

                file.read_exact(&mut page_buffer[..bytes_to_read])?;
                buffer.extend_from_slice(&page_buffer[..bytes_to_read]);
            }

            Ok(buffer)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))
        }
    }

    pub fn ls(&mut self) {
        for (name, entry) in &self.files {
            if entry.parent.as_deref() == Some(&self.current_path) {
                let suffix = match entry.file_type {
                    FileType::Directory => "/",
                    FileType::File => "",
                };
                let color = match entry.file_type {
                    FileType::Directory => "\x1b[34m",
                    FileType::File => "\x1b[33m", // yellow
                };
                let white = "\x1b[0m";
                self.stdio
                    .write(format!("{}{:?}{} {}{}\n", color, entry.file_type, white, name, suffix).as_bytes());
            }
        }
        self.stdio.print();
    }

    pub fn cd(&mut self, path: &str) -> std::io::Result<()> {
        let full_path = self.get_full_path(path);
        if path == "." {
            return Ok(());
        } else if path == ".." {
            if let Some(parent_path) = self
                .files
                .get(&self.current_path)
                .and_then(|entry| entry.parent.clone())
            {
                self.current_path = parent_path;
                return Ok(());
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No parent directory",
                ));
            }
        } else if let Some(entry) = self.files.get(&full_path) {
            if let FileType::Directory = entry.file_type {
                self.current_path = full_path;
                return Ok(());
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Not a directory",
                ));
            }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Directory not found",
            ));
        }
    }

    pub fn touch(&mut self, name: &str) -> std::io::Result<()> {
        self.create_file(
            name,
            FileType::File,
            self.current_user.get_user_name().as_str(),
            Some(vec![FilePermission::Readable, FilePermission::Writable]),
        )
    }

    pub fn mkdir(&mut self, name: &str) -> std::io::Result<()> {
        self.create_file(
            name,
            FileType::Directory,
            self.current_user.get_user_name().as_str(),
            Some(vec![FilePermission::Readable, FilePermission::Writable]),
        )
    }

    // pub fn _cat(&self, name: &str, path: &str) {
    //     match self.read_file(name, path) {
    //         Ok(data) => match String::from_utf8(data) {
    //             Ok(content) => println!("{}", content),
    //             Err(e) => eprintln!("Error converting file content to string: {}", e),
    //         },
    //         Err(e) => eprintln!("Error reading file {}: {}", name, e),
    //     }
    // }

    pub fn rm(&mut self, name: &str) -> std::io::Result<()> {
        let full_path = self.get_full_path(name);
        if let Some(entry) = self.files.remove(&full_path) {
            if entry.file_type == FileType::File {
                if self.current_user.get_user_name() != "root"
                    && entry.owned_user.unwrap() != self.current_user.get_user_name()
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "Permission denied",
                    ));
                }

                for page in entry.pages {
                    self.free_list.push_back(page);
                }
            } else if entry.file_type == FileType::Directory {
                let prefix = format!("{}/", full_path);
                let to_remove: Vec<_> = self
                    .files
                    .keys()
                    .filter(|k| k.starts_with(&prefix))
                    .cloned()
                    .collect();
                for key in to_remove {
                    self.rm(&key)?;
                }
            }
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File or directory not found",
            ))
        }
    }

    pub fn cp<P: AsRef<Path>>(
        &mut self,
        src_name: &str,
        dest_name: &str,
        path: P,
    ) -> std::io::Result<()> {
        let src_full_path = self.get_full_path(src_name);
        // Get the parent path of the destination file from the full path

        let (data, user, permissions) = {
            if let Some(src_entry) = self.files.get(&src_full_path) {
                (
                    self.read_file(src_name, &path)?,
                    <std::option::Option<std::string::String> as Clone>::clone(
                        &src_entry.owned_user,
                    )
                    .unwrap()
                    .to_string(),
                    src_entry.permission.clone(),
                )
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Source file not found",
                ));
            }
        };

        self.create_file(dest_name, FileType::File, user.as_str(), permissions)?;
        self.write_file(dest_name, &data, path)?;
        Ok(())
    }

    pub fn mv<P: AsRef<Path>>(&mut self, src_name: &str, dest_name: &str, path: P) -> std::io::Result<()> {
        self.cp(src_name, dest_name, path)?;
        self.rm(src_name)
    }

    pub fn is_file_exists(&self, name: &str) -> bool {
        self.files.contains_key(&self.get_full_path(name))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).truncate(true).open(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P, user: User) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut fs: FileSystem = serde_json::from_reader(file)?;
        fs.current_user = user;
        Ok(fs)
    }
}
