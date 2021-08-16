use crate::utils::*;
use clap::ArgMatches;
use std::error::Error;
use std::fs::{create_dir, File};
use std::io::prelude::*;
use std::iter::Iterator as _;
use std::path::Path;

pub fn hash_object(repo_path: &Path, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let stdin = std::io::stdin();

    let mut files: Vec<Box<dyn Read>> = if matches.is_present("use_stdin") {
        vec![Box::new(stdin.lock())]
    } else if let Some(files) = matches.values_of("file") {
        match files
            .map(|file| File::open(&file).and_then(|f| Ok(Box::new(f) as Box<dyn Read>)))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(files) => files,
            Err(e) => {
                eprintln!("{}", e);
                vec![]
            }
        }
    } else {
        println!("{}", matches.usage());
        vec![]
    };

    for f in files.iter_mut() {
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let (hash, compressed) =
            compress_object(ObjectType::Blob, &buf).expect("failed to hashing object.");
        let hex_hash: String = hash.iter().map(|x| format!("{:02x}", x)).collect();
        if matches.is_present("actually_write") {
            let index_dir = format!("{}/objects/{}", repo_path.to_str().unwrap(), &hex_hash[..2]);
            let file_path = format!("{}/{}", index_dir, &hex_hash[2..]);
            if !Path::new(&index_dir).exists() {
                create_dir(&index_dir)?;
            }
            File::create(file_path)?.write_all(&compressed)?;
        }
        println!("{}", hex_hash);
    }

    Ok(())
}
