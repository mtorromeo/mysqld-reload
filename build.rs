use scraper::{ElementRef, Html, Selector};
use std::{convert::TryFrom, env, fs, io::Write, path::Path};
use structopt::clap::Shell;

include!("src/cli.rs");

#[derive(Debug)]
struct VariableDefinition {
    name: String,
    vartype: VariableType,
}

impl VariableDefinition {
    fn struct_def(&self) -> String {
        format!(
            r#"    VariableDefinition {{
        name: {:?},
        vartype: VariableType::{:?},
    }}"#,
            self.name, self.vartype
        )
    }
}

impl<'a> TryFrom<ElementRef<'a>> for VariableDefinition {
    type Error = String;

    fn try_from(element: ElementRef<'a>) -> Result<VariableDefinition, Self::Error> {
        let mut dynamic = false;
        let mut name = None;
        let mut vartype = None;
        let name_selector = Selector::parse("code.literal > a.link").unwrap();
        let mut prop;

        for row in element.children() {
            if let Some(th) = row.first_child().and_then(ElementRef::wrap) {
                prop = th.inner_html();
                if let Some(td) = th.next_siblings().find_map(ElementRef::wrap) {
                    match &prop[..] {
                        "Dynamic" => dynamic = td.inner_html() == "Yes",
                        "Type" => vartype = Some(td.inner_html()),
                        "System Variable" => {
                            if let Some(a) = td.select(&name_selector).next() {
                                name = Some(a.inner_html());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        match (dynamic, name, vartype) {
            (true, Some(name), Some(vartype)) => Ok(VariableDefinition {
                name,
                vartype: VariableType::try_from(&vartype[..]).unwrap(),
            }),
            _ => Err("Missing informations or non-dynamic variable".to_string()),
        }
    }
}

#[derive(Debug)]
enum VariableType {
    Boolean,
    String,
    Integer,
    Numeric,
    File,
    Directory,
    Enum,
    Set,
}

impl<'a> TryFrom<&'a str> for VariableType {
    type Error = String;

    fn try_from(strtype: &'a str) -> Result<VariableType, Self::Error> {
        match strtype {
            "Boolean" => Ok(VariableType::Boolean),
            "String" => Ok(VariableType::String),
            "Integer" => Ok(VariableType::Integer),
            "Numeric" => Ok(VariableType::Numeric),
            "File name" => Ok(VariableType::File),
            "Directory name" => Ok(VariableType::Directory),
            "Set" => Ok(VariableType::Set),
            "Enumeration" => Ok(VariableType::Enum),
            t => Err(format!("Unrecognized type: {}", t)),
        }
    }
}

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };
    let vardef_path = Path::new(&outdir).join("mysql_system_vardef.rs");
    let mut vardef_file = fs::File::create(&vardef_path).unwrap();
    let mut vardefs = Vec::new();

    let informal_tables_sel =
        Selector::parse("li.listitem > div.informaltable > table > tbody").unwrap();

    for html in &[
        "server-system-variables.html",
        "replication-options-binary-log.html",
    ] {
        println!("cargo:rerun-if-changed={}", html);

        let contents = fs::read_to_string(html).unwrap();
        let document = Html::parse_fragment(&contents);

        vardefs.extend(
            document
                .select(&informal_tables_sel)
                .filter_map(|e| VariableDefinition::try_from(e).ok()),
        );
    }

    vardefs.sort_by(|a, b| a.name.cmp(&b.name));
    vardef_file
        .write_all(
            format!(
                "pub const MYSQL_SYSTEM_VARIABLES: [VariableDefinition; {}] = [\n{}\n];\n",
                vardefs.len(),
                vardefs
                    .into_iter()
                    .map(|vardef| vardef.struct_def())
                    .collect::<Vec<String>>()
                    .join(",\n")
            )
            .as_bytes(),
        )
        .unwrap();

    let mut app = Opts::clap();
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Fish, &outdir);
}
