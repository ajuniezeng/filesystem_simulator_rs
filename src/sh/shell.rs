use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use crate::{
    fs::{filesystem::FileSystem, stdio::Stdio},
    user::{User, Users},
};

#[derive(Debug, Clone)]
pub struct Shell {
    current_user: User,
    stdio: Stdio,
}

impl Shell {
    pub fn init(user: String) -> std::io::Result<()> {
        let users_path = "users.json";

        let mut users = if Path::new(users_path).exists() {
            Users::load(users_path)?
        } else {
            File::create(users_path)?;
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
        
        let mut shell = Self { current_user: user, stdio: Stdio::new()};

        shell.run()?;

        Ok(())
    }

    fn run(&mut self) -> std::io::Result<()> {
        let fs_path = "filesystem.json";
        let container_path = "container.bin";

        if !Path::new(container_path).exists() {
            File::create(container_path)?;
        }

        let mut fs = if Path::new(fs_path).exists() {
            FileSystem::load(fs_path, self.current_user.clone())?
        } else {
            File::create(fs_path)?;
            FileSystem::new(self.current_user.clone())
        };

        let mut pointer = ">";
        if self.current_user.get_user_name() == "root" {
            pointer = "$";
        }

        println!("Welcome! {}", self.current_user.get_user_name());

        loop {
            print!(
                "{} {} {} ",
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
            let args = &parts[1..];

            // let mut input_redirect: Option<String> = None;
            // let mut output_redirect: Option<String> = None;

            // for (i, part) in parts.iter().enumerate() {
            //     if *part == "<" {
            //         input_redirect = Some(parts[i + 1].to_string());
            //         args = &parts[1..i];
            //     } else if *part == ">" {
            //         output_redirect = Some(parts[i + 1].to_string());
            //         args = &parts[1..i];
            //     }
            // }

            match command {
                "cd" => {
                    if args.len() > 0 {
                        if let Err(e) = fs.cd(args[0]) {
                            self.stdio.error(format!("Error: {}", e).as_bytes());
                            self.stdio.print_error();
                            continue;
                        };
                    } else {
                        self.stdio.write("Usage: cd <path>".as_bytes());
                        self.stdio.print();
                    }
                }
                "ls" => {
                    fs.ls();
                }
                "touch" => {
                    if args.len() > 0 {
                        if let Err(e) = fs.touch(args[0]) {
                            self.stdio.error(format!("Error: {}", e).as_bytes());
                            self.stdio.print_error();
                            continue;
                        };
                    } else {
                        self.stdio.write("Usage: touch <filename>".as_bytes());
                        self.stdio.print();
                    }
                }
                "cat" => {
                    if !self.current_user.permissions.can_read {
                        self.stdio.write("Permission denied".as_bytes());
                        self.stdio.print();
                        continue;
                    }

                    if args.len() > 0 {
                        if let Err(e) = self.stdio.read_file(args[0], container_path, &fs) {
                            self.stdio.error(format!("Error: {}", e).as_bytes());
                            self.stdio.print_error();
                            continue;
                        };
                        self.stdio.print();
                    } else {
                        self.stdio.write("Usage: cat <filename>".as_bytes());
                        self.stdio.print();
                    }
                }
                "mkdir" => {
                    if args.len() > 0 {
                        if let Err(e) = fs.mkdir(args[0]) {
                            self.stdio.error(format!("Error: {}", e).as_bytes());
                            self.stdio.print_error();
                            continue;
                        };
                    } else {
                        self.stdio.write("Usage: mkdir <directory>".as_bytes());
                        self.stdio.print();
                    }
                }
                "rm" => {
                    if args.len() > 0 {
                        if let Err(e) = fs.rm(args[0]) {
                            self.stdio.error(format!("Error: {}", e).as_bytes());
                            self.stdio.print_error();
                            continue;
                        };
                    } else {
                        self.stdio.write("Usage: rm <filename>".as_bytes());
                        self.stdio.print();
                    }
                }
                "mv" => {
                    if args.len() > 1 {
                        if let Err(e) = fs.mv(args[0], args[1], container_path) {
                            self.stdio.error(format!("Error: {}", e).as_bytes());
                            self.stdio.print_error();
                            continue;
                        };
                    }
                }
                "cp" => {
                    if args.len() > 1 {
                        if let Err(e) = fs.cp(args[0], args[1], container_path) {
                            self.stdio.error(format!("Error: {}", e).as_bytes());
                            self.stdio.print_error();
                            continue;
                        };
                    } else {
                        self.stdio.write("Usage: cp <source> <destination>".as_bytes());
                        self.stdio.print();
                    }
                }
                "echo" => {
                    if args.contains(&">") {
                        let data = args.split(|&x| x == ">").collect::<Vec<_>>();
                        if data.len() != 2 {
                            self.stdio.write("Usage: echo <data> [> <filename>]".as_bytes());
                            self.stdio.print();
                            continue;
                        } else {
                            let name = data[1].split(|&x| x == " ").collect::<Vec<_>>();
                            for i in name {
                                let i = i.join(" ");
                                if i != "" {
                                    if !fs.is_file_exists(i.as_str()) {
                                        if let Err(e) = fs.touch(i.as_str()) {
                                            self.stdio.error(format!("Error: {}", e).as_bytes());
                                            self.stdio.print_error();
                                            continue;
                                        };
                                    }
                                    if let Err(e) = fs.write_file(i.as_str(), data[0].join(" ").as_bytes(), container_path) {
                                        self.stdio.error(format!("Error: {}", e).as_bytes());
                                        self.stdio.print_error();
                                        continue
                                    };
                                }
                            }
                        }
                    } else {
                        let data = args.join(" ");
                        self.stdio.write(data.as_bytes());
                        self.stdio.print();
                    }
                }
                "exit" => {
                    fs.save(fs_path)?;
                    break;
                }
                _ => {
                    self.stdio.write("Command not found".as_bytes());
                    self.stdio.print();
                }
            }

            // if let Some(input_file) = input_redirect {
            //     self.stdio.read_file(&input_file, container_path, &fs)?;
            // }

            // if let Some(output_file) = output_redirect {
            //     self.stdio.write_file(&output_file, container_path, &mut fs)?;
            // }
        }

        Ok(())
    }

    // fn report_error(&mut self, error: &std::io::Error) {
    //     // Use stdio or another method to output error to the shell
    //     self.stdio.input(format!("Error: {}\n", error).as_bytes());
    //     self.stdio.print();
    // }
}
