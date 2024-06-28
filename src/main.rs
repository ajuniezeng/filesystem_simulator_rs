use clap::{Arg, ArgAction, Command};
use sh::shell::Shell;

mod fs;
mod sh;
mod user;

fn main() {
    let matches = Command::new("Linux Filesystem Simulator")
        .version("0.1.0")
        .author("AjunieZeng <ajuniezeng@gmail.com>")
        .about("Simulates a basic Linux filesystem")
        .arg(
            Arg::new("user")
                .short('u')
                .long("user")
                .action(ArgAction::Set)
                .num_args(1)
                .help("Specify the user"),
        )
        .get_matches();

    let user = matches
        .get_one::<String>("user")
        .unwrap_or(&"root".to_string())
        .to_string();
    
    if let Err(e) = Shell::init(user) {
        eprintln!("{}", e);
    } 
}
