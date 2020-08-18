use clap::ArgMatches;
use std::error::Error;
use std::path::Path;

pub fn hash_object(repo_path: &Path, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    use crate::utils::*;
    use std::fs::{create_dir, File};
    use std::io::prelude::*;
    use std::iter::Iterator as _;

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

pub fn cat_file(repo_path: &Path, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::iter::Iterator as _;

    let object = matches.value_of("object").ok_or("object not specified")?;
    // let object_type = {
    //     use crate::utils::ObjectType::*;
    //     match matches.value_of("object_type") {
    //         Some("blob") => Ok(Some(Blob)),
    //         Some("tree") => Ok(Some(Tree)),
    //         Some("commit") => Ok(Some(Commit)),
    //         Some("tag") => Ok(Some(Tag)),
    //         _ if matches.is_present("pretty") => Ok(None),
    //         _ => Err("please specify object type"),
    //     }
    // }?;
    let file_path = format!(
        "{}/objects/{}/{}",
        repo_path.to_str().unwrap(),
        &object[..2],
        &object[2..]
    );
    let mut object_body: Vec<u8> = Vec::new();
    File::open(&file_path)?.read_to_end(&mut object_body)?;

    let decompressed_object = &crate::utils::decompress_object(&object_body)?;
    let decompressed_str = String::from_utf8_lossy(decompressed_object);

    let mut decompressed_str = decompressed_str.splitn(2, ' ');
    let _actual_object_type = decompressed_str.next().unwrap();

    let mut decompressed_str = decompressed_str.next().unwrap().splitn(2, '\0');
    let _actual_object_size = decompressed_str.next().unwrap();

    let decompressed_str = decompressed_str.next().unwrap();

    println!("{}", decompressed_str);

    Ok(())
}
