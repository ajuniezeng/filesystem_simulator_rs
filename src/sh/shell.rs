use std::io::{self, Write};
use std::path::Path;

use crate::{
    fs::{filesystem::FileSystem, stdio::Stdio},
    user::{User, Users},
};

#[derive(Debug, Clone)]
pub struct Shell {
    current_user: User,
}

impl Shell {
    pub fn init(user: String) -> std::io::Result<()> {
        let users_path = "users.json";

        let mut users = if Path::new(users_path).exists() {
            Users::load(users_path)?
        } else {
            Users::new()
        };

        let user = if users.get_users().is_empty() {
            if user == "root" {
                let user = User::new("root", true, true, true);
                users.add_user(user.clone());
                user
            } else {
                let user = User::new(&user, true, true, false);
                users.add_user(user.clone());
                user
            }
        } else {
            let mut specified_user: Option<User> = None;
            for i in users.get_users() {
                if i.get_user_name() == user {
                    specified_user = Some(i.clone());
                    break;
                } else {
                    continue;
                }
            }
            if let Some(user) = specified_user {
                user
            } else {
                let user = User::new(&user, true, true, false);
                users.add_user(user.clone());
                user
            }
        };

        users.save(users_path)?;
        
        let mut shell = Self { current_user: user };

        shell.run()?;

        Ok(())
    }

    fn run(&mut self) -> std::io::Result<()> {
        let fs_path = "filesystem.json";
        let container_path = "container.bin";

        let mut fs = if Path::new(fs_path).exists() {
            FileSystem::load(fs_path)?
        } else {
            FileSystem::new(self.current_user.clone())
        };

        let mut stdio = Stdio::new();
        let mut pointer = ">";
        if self.current_user.get_user_name() == "root" {
            pointer = "$";
        }

        println!("Welcome! {}", self.current_user.get_user_name());

        loop {
            print!(
                "{} {}{}",
                self.current_user.get_user_name(),
                fs.current_path,
                pointer
            );
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let command = parts[0];
            let mut args = &parts[1..];

            let mut input_redirect: Option<String> = None;
            let mut output_redirect: Option<String> = None;

            for (i, part) in parts.iter().enumerate() {
                if *part == "<" {
                    input_redirect = Some(parts[i + 1].to_string());
                    args = &parts[1..i];
                } else if *part == ">" {
                    output_redirect = Some(parts[i + 1].to_string());
                    args = &parts[1..i];
                }
            }

            match command {
                "cd" => {
                    if args.len() > 0 {
                        fs.cd(args[0])?;
                    } else {
                        println!("Usage: cd <path>");
                    }
                }
                "ls" => {
                    fs.ls();
                }
                "touch" => {
                    if args.len() > 0 {
                        fs.touch(args[0])?;
                    } else {
                        println!("Usage: touch <filename>")
                    }
                }
                "cat" => {
                    if !self.current_user.permissions.can_read {
                        println!("Permission denied")
                    }

                    if args.len() > 0 {
                        stdio.read_file(args[0], container_path, &fs)?;
                        stdio.print();
                    } else {
                        println!("Usage: cat <filename>");
                    }
                }
                "mkdir" => {
                    if args.len() > 0 {
                        fs.mkdir(args[0])?;
                    } else {
                        println!("Usage: mkdir <dirname>");
                    }
                }
                "rm" => {
                    if args.len() > 0 {
                        fs.rm(args[0])?;
                    } else {
                        println!("Usage: rm <filename>");
                    }
                }
                "mv" => {
                    if args.len() > 1 {
                        fs.mv(args[0], args[1])?
                    }
                }
                "cp" => {
                    if args.len() > 1 {
                        fs.cp(args[0], args[1], container_path)?;
                    } else {
                        println!("Usage: cp <source> <destination>");
                    }
                }
                "exit" => {
                    fs.save(fs_path)?;
                    break;
                }
                _ => {
                    println!("{}: command not found", command);
                }
            }

            if let Some(input_file) = input_redirect {
                stdio.read_file(&input_file, container_path, &fs)?;
            }

            if let Some(output_file) = output_redirect {
                stdio.write_file(&output_file, container_path, &mut fs)?;
            }
        }

        Ok(())
    }
}
