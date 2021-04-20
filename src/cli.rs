use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(global_settings(&[AppSettings::ColoredHelp]), author, about)]
pub struct Opts {
    /// MySQL server configuration file
    #[structopt(short, long, default_value = "/etc/my.cnf")]
    pub cnf: PathBuf,

    /// Print the SQL statements to stdout
    #[structopt(short, long)]
    pub verbose: bool,

    /// Do not apply values
    #[structopt(short, long)]
    pub dry_run: bool,
}
