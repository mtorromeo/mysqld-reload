mod cli;
mod mysql_variables;

use cli::Opts;
use configparser::ini::Ini;
use mysql::params;
use mysql::prelude::*;
use mysql_variables::VariableDefinition;
use std::fs::File;
use std::io::Read;
use std::{collections::HashMap, path::Path};
use structopt::StructOpt;
use users::get_current_username;

#[derive(Debug)]
struct Variable {
    name: String,
    value: String,
}

impl Variable {
    fn definition(&self) -> Option<&VariableDefinition> {
        VariableDefinition::get(&self.name)
    }
}

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
type MyCnfSection = HashMap<String, Option<String>>;

#[derive(Debug)]
struct LoginSettings {
    user: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: u16,
    socket: Option<String>,
}

impl Default for LoginSettings {
    fn default() -> Self {
        Self {
            user: get_current_username().and_then(|u| u.into_string().ok()),
            password: None,
            host: None,
            port: 3306,
            socket: None,
        }
    }
}

fn read_login_mycnf(file: &Path) -> DynResult<LoginSettings> {
    let mycnf = read_mycnf(&file)?;

    let mut login = LoginSettings {
        user: None,
        password: None,
        host: None,
        port: 3306,
        socket: None,
    };

    let client_sections = ["mysql", "client"];
    for section in &client_sections {
        if let Some(options) = mycnf.get(&section.to_string()) {
            if let Some(Some(user)) = options.get("user") {
                login.user = Some(user.clone());
            }

            if let Some(Some(pass)) = options.get("password") {
                login.password = Some(pass.clone());
            }

            if let Some(Some(host)) = options.get("host") {
                login.host = Some(host.clone());
            }

            if let Some(Some(port)) = options.get("port") {
                if let Ok(port) = port.parse() {
                    login.port = port;
                }
            }

            if let Some(Some(socket)) = options.get("socket") {
                login.socket = Some(socket.clone());
            }
        }
    }

    Ok(login)
}

fn read_mycnf(file: &Path) -> DynResult<HashMap<String, MyCnfSection>> {
    let mut config_file = File::open(&file)?;
    let mut s = String::new();
    config_file.read_to_string(&mut s)?;

    let mut ini = Ini::new();
    ini.read(s)?;
    Ok(ini.get_map().unwrap_or_default())
}

fn normalize_conf(config: &HashMap<String, Option<String>>) -> HashMap<String, String> {
    let mut normalized_config = HashMap::new();
    for (k, v) in config.iter() {
        let mut normalized_k = k.to_lowercase().replace("-", "_");
        let mut v = v.clone().unwrap_or_else(|| "ON".to_string());

        // unquote strings
        let first_char = &v[0..1];
        if v.len() > 1 && v.ends_with(first_char) && (first_char == "'" || first_char == "\"") {
            v.remove(0);
            v.remove(v.len() - 1);
        }

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
    definition: &VariableDefinition,
    verbose: bool,
    dry_run: bool,
) -> Result<(), mysql::Error> {
    let name = mysql_escape_identifier(name);

    let quote_value = !matches!(
        definition.vartype,
        mysql_variables::VariableType::Boolean
            | mysql_variables::VariableType::Integer
            | mysql_variables::VariableType::Numeric
    );
    let stmt = format!(
        "SET GLOBAL {} = {};",
        name,
        if quote_value { ":value" } else { value }
    );

    if verbose {
        println!("{}", stmt.replacen(":value", value, 1));
    }

    if !dry_run {
        let _: Vec<String> = if quote_value {
            conn.exec(
                stmt,
                params! {
                    "value" => value,
                },
            )?
        } else {
            conn.query(stmt)?
        };
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

    let defaults_file = opts.defaults_file.or_else(|| {
        dirs::home_dir().map(|mut home| {
            home.push(".my.cnf");
            home
        })
    });

    let mycnf = match defaults_file {
        Some(defaults_file) if !opts.no_defaults => {
            read_login_mycnf(&defaults_file).unwrap_or_default()
        }
        _ => LoginSettings::default(),
    };

    let myopts = mysql::OptsBuilder::new()
        .user(opts.user.or(mycnf.user))
        .pass(opts.password.or(mycnf.password))
        .ip_or_hostname(opts.host.or(mycnf.host))
        .tcp_port(opts.port.unwrap_or(mycnf.port))
        .socket(
            opts.socket
                .map(|s| s.to_string_lossy().to_string())
                .or(mycnf.socket),
        );

    let mut conn = mysql::Conn::new(myopts)?;
    let mysqld_variables = conn.query_map("SHOW GLOBAL VARIABLES", |(name, value)| Variable {
        name,
        value,
    })?;

    for variable in mysqld_variables.iter() {
        if let Some(definition) = variable.definition() {
            if let Some(option) = config.get(&variable.name) {
                if let Some(new_normalized) = definition.same(option, &variable.value) {
                    // if opts.verbose {
                    //     println!("{:?} -> {:?}", variable, definition.vartype);
                    // }
                    mysql_set_var(
                        &mut conn,
                        &variable.name,
                        &new_normalized,
                        definition,
                        opts.verbose,
                        opts.dry_run,
                    )?;
                }
            }
        }
    }

    Ok(())
}
