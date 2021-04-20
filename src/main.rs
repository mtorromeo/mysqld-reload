mod dynamic_system_variables;

use configparser::ini::Ini;
use dynamic_system_variables::DYNAMIC_SYSTEM_VARIABLES;
use mysql::params;
use mysql::prelude::*;
use std::fs::File;
use std::io::Read;
use std::{collections::HashMap, path::Path, path::PathBuf};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(global_settings(&[AppSettings::ColoredHelp]), author, about)]
struct Opts {
    /// MySQL server configuration file
    #[structopt(short, long, default_value = "/etc/my.cnf")]
    cnf: PathBuf,

    /// Print the SQL statements to stdout
    #[structopt(short, long)]
    verbose: bool,

    /// Do not apply values
    #[structopt(short, long)]
    dry_run: bool,
}

struct Variable {
    name: String,
    value: String,
}

impl Variable {
    fn is_bool(&self) -> bool {
        let upper = self.value.to_uppercase();
        upper == "ON" || upper == "YES" || upper == "OFF" || upper == "NO"
    }

    fn as_bool(&self) -> bool {
        let upper = self.value.to_uppercase();
        upper == "ON" || upper == "YES"
    }
}

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
type MyCnfSection = HashMap<String, Option<String>>;

fn read_defaultscnf() -> DynResult<Option<HashMap<String, MyCnfSection>>> {
    match dirs::home_dir() {
        None => Ok(None),
        Some(mut mycnf) => {
            mycnf.push(".my.cnf");
            if !mycnf.is_file() {
                return Ok(None);
            }
            Ok(Some(read_mycnf(&mycnf)?))
        }
    }
}

fn read_mycnf(file: &Path) -> DynResult<HashMap<String, MyCnfSection>> {
    let mut config_file = File::open(&file)?;
    let mut s = String::new();
    config_file.read_to_string(&mut s)?;

    let mut ini = Ini::new();
    ini.read(s)?;
    Ok(ini.get_map().expect("Config file was read"))
}

fn normalize_conf(config: &HashMap<String, Option<String>>) -> HashMap<String, String> {
    let mut normalized_config = HashMap::new();
    for (k, v) in config.iter() {
        let mut normalized_k = k.to_lowercase().replace("-", "_");
        let mut v = v.clone().unwrap_or_else(|| "ON".to_string());
        if normalized_k.starts_with("skip_") {
            normalized_k = normalized_k.replacen("skip_", "", 1);
            v = "OFF".to_string();
        }
        normalized_config.insert(normalized_k, v);
    }
    normalized_config
}

fn mysql_escape_identifier(name: &str) -> String {
    format!("`{}`", name.replace("`", "``"))
}

fn mysql_set_var(
    conn: &mut mysql::Conn,
    name: &str,
    value: &str,
    verbose: bool,
    dry_run: bool,
) -> Result<(), mysql::Error> {
    let name = mysql_escape_identifier(name);
    let stmt = format!("SET GLOBAL {} = :value;", name);

    if verbose {
        println!("{}", stmt.replacen(":value", value, 1));
    }

    if !dry_run {
        let _: Vec<String> = conn.exec(
            stmt,
            params! {
                "value" => value,
            },
        )?;
    }

    Ok(())
}

fn main() -> DynResult<()> {
    let opts = Opts::from_args();

    let config = read_mycnf(&opts.cnf)?;
    let config = match config.get("mysqld") {
        Some(mysqld) => mysqld,
        None => return Ok(()),
    };

    let config = normalize_conf(config);

    let mut myopts = mysql::OptsBuilder::new();
    if let Some(mycnf) = read_defaultscnf()? {
        let client_sections = ["mysql", "client"];
        for section in &client_sections {
            if let Some(options) = mycnf.get(&section.to_string()) {
                if let Some(Some(user)) = options.get("user") {
                    myopts = myopts.user(Some(user));
                }
                if let Some(Some(user)) = options.get("password") {
                    myopts = myopts.pass(Some(user));
                }
            }
        }
    }

    let mut conn = mysql::Conn::new(myopts)?;
    let mysqld_variables = conn.query_map("SHOW GLOBAL VARIABLES", |(name, value)| Variable {
        name,
        value,
    })?;

    for variable in mysqld_variables.iter() {
        if DYNAMIC_SYSTEM_VARIABLES
            .binary_search_by(|v| (*v).cmp(variable.name.as_str()))
            .is_ok()
        {
            if let Some(option) = config.get(&variable.name) {
                if variable.is_bool() {
                    let v = option.to_uppercase();
                    let v = v == "YES" || v == "ON" || v == "1";
                    if v != variable.as_bool() {
                        mysql_set_var(
                            &mut conn,
                            &variable.name,
                            if v { "ON" } else { "OFF" },
                            opts.verbose,
                            opts.dry_run,
                        )?;
                    }
                } else if &variable.value != option {
                    mysql_set_var(
                        &mut conn,
                        &variable.name,
                        option,
                        opts.verbose,
                        opts.dry_run,
                    )?;
                }
            }
        }
    }

    Ok(())
}
