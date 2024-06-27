use std::{
    fs::{File, OpenOptions},
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Users {
    users: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    username: String,
    pub permissions: Permissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub can_read: bool,
    pub can_write: bool,
    pub can_execute: bool,
}

impl User {
    pub fn new(username: &str, can_read: bool, can_write: bool, can_execute: bool) -> Self {
        Self {
            username: username.to_string(),
            permissions: Permissions {
                can_read,
                can_write,
                can_execute,
            },
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new("root", true, true, true)
    }
}

impl User {
    pub fn get_user_name(&self) -> String {
        return self.username.to_string();
    }
}

impl Users {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.push(user);
    }

    pub fn get_users(&self) -> &Vec<User> {
        return &self.users;
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).truncate(true).open(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let user: Users = serde_json::from_reader(file)?;
        Ok(user)
    }
}
