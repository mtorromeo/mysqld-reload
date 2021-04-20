#[derive(Debug)]
pub struct VariableDefinition {
    pub name: &'static str,
    pub vartype: VariableType,
}

#[derive(Debug)]
pub enum VariableType {
    Boolean,
    String,
    Integer,
    Numeric,
    File,
    Directory,
    Enum,
    Set,
}

const SIZE_SUFFIXES: [&str; 6] = ["K", "M", "G", "T", "P", "E"];

impl VariableDefinition {
    fn normalize(&self, value: &str) -> String {
        let mut value = match self.vartype {
            VariableType::Boolean
            | VariableType::Integer
            | VariableType::Numeric
            | VariableType::Set
            | VariableType::Enum => value.to_uppercase(),
            _ => value.to_owned(),
        };

        value = match self.vartype {
            VariableType::Boolean if value == "YES" || value == "1" => "ON".to_owned(),
            VariableType::Boolean if value == "ON" => value,
            VariableType::Boolean => "OFF".to_owned(),
            VariableType::Integer if value.len() > 0 => {
                let suffix = &value[value.len() - 1..value.len()];
                match SIZE_SUFFIXES.iter().position(|&x| x == suffix) {
                    Some(pos) => match &value[0..value.len() - 1].parse::<i32>() {
                        Ok(num) => format!("{}", num * 1024i32.pow((pos as u32) + 1)),
                        _ => value,
                    },
                    _ => value,
                }
            }
            _ => value,
        };

        value
    }

    pub fn same(&self, a: &str, b: &str) -> bool {
        let a = self.normalize(a);
        let b = self.normalize(b);
        a == b
    }
}

include!(concat!(env!("OUT_DIR"), "/mysql_system_vardef.rs"));
