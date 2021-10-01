use clap::clap_app;
use std::path::Path;

mod commands;
mod errors;
mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // v3.0でハイフンが使えるようになるらしい…
    let matches = clap_app!(git =>
        (version: "0.1")
        (@setting SubcommandRequiredElseHelp)
        (@arg repo: -r --repo +takes_value "repository path")
        (@subcommand hash_object =>
            (about: "hash given objects")
            (@arg actually_write: -w "actually write object to database")
            (@group input_group =>
                (@arg use_stdin: --stdin "use stdin as input")
                (@arg file: ... "target file")
            )
        )
        (@subcommand cat_file =>
            (about: "cat object")
            // (@group object_type =>
            //     (@arg pretty: -p "pretty print object by detecting object type")
            //     (@arg obj_type: "object type")
            // )
            (@arg object: "target object")
        )
        (@subcommand read_index =>
            (about: "read from index and show it (original command)")
        )
    )
    .get_matches();

    let mut repo_path = Path::new(".git");
    loop {
        if repo_path.exists() && repo_path.is_dir() {
            break;
        }
        if repo_path.parent().is_none() {
            panic!("this is not a git repository.");
        }
        repo_path = repo_path.parent().unwrap();
    }

    if let Some(matches) = matches.subcommand_matches("hash_object") {
        commands::hash_object::hash_object(&repo_path, &matches)?
    } else if let Some(matches) = matches.subcommand_matches("cat_file") {
        commands::cat_file::cat_file(&repo_path, &matches)?
    } else if matches.subcommand_matches("read_index").is_some() {
        commands::read_index::read_index(&repo_path)?
    }

    Ok(())
}
