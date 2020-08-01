use clap::{clap_app, ArgMatches};

fn main() {
    let matches: ArgMatches = clap_app!(app =>
        (@setting SubcommandRequiredElseHelp)
        (version: "0.1")
        (@subcommand commit =>
            (@arg message: -m * +takes_value "commit message") // エディタ開くのめんどくさいので...
        )
    )
    .get_matches();

    if let Some(opt) = matches.subcommand_matches("commit") {
        std::process::Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(opt.value_of("message").unwrap())
            .spawn();
    } else {
        matches.usage();
    }
}
