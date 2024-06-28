# Simple Filesystem Simulator

This is a course project by [Ajunie Zeng](ajuniezeng@gmail.com)

## How to run

For those who haven't installed Rust, please install it first by following the instructions on the [official website](https://www.rust-lang.org/tools/install).

And then run the following command in the root directory of the project:

```shell
cargo run -- [username]
```

If you input a username that doesn't exist, the program will create a new user with the given username. Default username is `root`.

## Features

This project has implemented the following commands:

- [x] `ls` - List directory contents
- [x] `cd` - Change the shell working directory
- [x] `mkdir` - Make directories
- [x] `rm` - Remove files or directories
- [x] `touch` - Create an empty file
- [x] `cat` - Print the file on the standard output
- [x] `echo` - Display a line of text or insert the text into a file
- [x] `cp` - Copy files and directories
- [x] `mv` - Move files and directories

And the following features:

- [x] Multiple users
- [x] Simple Permissions
- [x] Tree structure of the filesystem

When you run the program, it will create `users.json`, `filesystem.json` and `container.bin` in the root directory of the project. The `users.json` file stores the information of the users, the `filesystem.json` file stores the tree structure of the filesystem, and the `container.bin` file stores the content of the files.

## License

See [LICENSE](LICENSE) for more details.
