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

    /// Connect to host
    #[structopt(short = "H", long)]
    pub host: Option<String>,

    /// User for login if not current user
    #[structopt(short, long)]
    pub user: Option<String>,

    /// Password to use when connecting to server
    #[structopt(short, long)]
    pub password: Option<String>,

    /// Port number to use for connection
    #[structopt(short = "P", long)]
    pub port: Option<u16>,

    /// The socket file to use for connection
    #[structopt(short = "S", long)]
    pub socket: Option<PathBuf>,

    /// Don't read default options from any option file, except for login file
    #[structopt(long)]
    pub no_defaults: bool,

    /// Only read default options from the given file
    #[structopt(long)]
    pub defaults_file: Option<PathBuf>,
}
