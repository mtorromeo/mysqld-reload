use std::env;
use structopt::clap::Shell;

include!("src/cli.rs");

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };

    let mut app = Opts::clap();
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Fish, &outdir);
}
