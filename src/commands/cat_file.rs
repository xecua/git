use anyhow::Context;
use clap::ArgMatches;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Iterator as _;
use std::path::Path;

pub fn cat_file(repo_path: &Path, matches: &ArgMatches) -> anyhow::Result<()> {
    let object = matches.value_of("object").context("object not specified")?;
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
