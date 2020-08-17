use clap::{clap_app, ArgMatches};
use git::utils::ObjectType;
use std::fs::{create_dir, File};
use std::io::{self, prelude::*};
use std::iter::Iterator as _;
use std::path::Path;

fn main() -> io::Result<()> {
    let stdin = std::io::stdin();

    // v3.0でハイフンが使えるようになるらしい…
    let matches: ArgMatches = clap_app!(git =>
        (@setting SubcommandRequiredElseHelp)
        (version: "0.1")
        (@arg repo: -r --repo +takes_value "repository path")
        (@subcommand hash_object =>
            (about: "hash given objects")
            (@arg actually_write: -w "actually write object to database")
            (@group input_group =>
                (@arg use_stdin: --stdin "use stdin as input")
                (@arg file: ... "target file")
            )
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

    if let Some(matches) = matches.subcommand_matches("hash_object") {
        let mut files: Vec<Box<dyn Read>> = if matches.is_present("use_stdin") {
            vec![Box::new(stdin.lock())]
        } else {
            let files = matches.values_of("file");
            if files.is_none() {
                println!("{}", matches.usage());
                vec![]
            } else {
                match files
                    .unwrap()
                    .map(|file| File::open(&file).and_then(|f| Ok(Box::new(f) as Box<dyn Read>)))
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(files) => files,
                    Err(e) => {
                        eprintln!("{}", e);
                        vec![]
                    }
                }
            }
        };
        for f in files.iter_mut() {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            let (hash, compressed) =
                git::utils::hash_object(ObjectType::Blob, &buf).expect("failed to hashing object.");
            let hex_hash: String = hash.iter().map(|x| format!("{:02x}", x)).collect();
            if matches.is_present("actually_write") {
                let index_dir = format!("{}/objects/{}", repo.to_str().unwrap(), &hex_hash[..2]);
                let file_path = format!("{}/{}", index_dir, &hex_hash[2..]);
                if !Path::new(&index_dir).exists() {
                    create_dir(&index_dir)?;
                }
                File::create(file_path)?.write_all(&compressed).unwrap();
            }
            println!("{}", hex_hash);
        }
    }

    Ok(())
}
