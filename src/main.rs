use clap::{clap_app, ArgMatches};
use git::utils::ObjectType;
use std::fs::{create_dir, File};
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let matches: ArgMatches = clap_app!(git =>
        (@setting SubcommandRequiredElseHelp)
        (version: "0.1")
        (@arg repo: -r --repo +takes_value "repository path")
        (@subcommand hash-object =>
            (@arg help: -h --help "display help")
            (@arg use_stdin: --stdin "use stdin as input")
            (@arg actually_write: -w "if actually write object to database")
            (@arg file: * ... conflicts_with[use_stdin] "target file")
        )
    )
    .get_matches();

    let mut repo = Path::new(".git");
    loop {
        if repo.exists() && repo.is_dir() {
            break;
        }
        if repo.parent().is_none() {
            panic!("this is not a git repository.");
        }
        repo = repo.parent().unwrap();
    }

    if let Some(matches) = matches.subcommand_matches("hash-object") {
        if matches.is_present("help") {
            println!("{}", matches.usage());
        } else if matches.is_present("use_stdin") {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap();
            let (hash, compressed) = git::utils::hash_object(ObjectType::Blob, buf[..].as_bytes())
                .expect("failed to hashing object.");
            if matches.is_present("actually_write") {
                let index_dir = format!(
                    "{}/{}",
                    repo.to_str().unwrap(),
                    String::from_utf8(hash[..2].to_vec()).unwrap()
                );
                let file_path = format!(
                    "{}/{}",
                    index_dir,
                    String::from_utf8(hash[2..].to_vec()).unwrap()
                );
                create_dir(Path::new(&index_dir));
                File::open(Path::new(&file_path))
                    .unwrap()
                    .write_all(&compressed)
                    .unwrap();
            }
            println!("{}", String::from_utf8(hash));
        }
    }
}
