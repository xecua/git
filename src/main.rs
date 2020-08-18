use clap::{clap_app, ArgMatches};
use std::path::Path;

fn main() -> std::io::Result<()> {
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
        git::commands::hash_object(&repo_path, &matches)?;
    }

    Ok(())
}
