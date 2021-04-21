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
    pub fn get(name: &str) -> Option<&Self> {
        match MYSQL_SYSTEM_VARIABLES.binary_search_by(|v| v.name.cmp(name)) {
            Ok(pos) => Some(&MYSQL_SYSTEM_VARIABLES[pos]),
            Err(_) => None,
        }
    }

    fn normalize(&self, value: &str) -> String {
        let value = match self.vartype {
            VariableType::Boolean
            | VariableType::Integer
            | VariableType::Numeric
            | VariableType::Set
            | VariableType::Enum => value.to_uppercase(),
            _ => value.to_owned(),
        };

        match self.vartype {
            VariableType::Boolean if value == "YES" || value == "TRUE" || value == "1" => {
                "ON".to_owned()
            }
            VariableType::Boolean if value == "ON" => value,
            VariableType::Boolean => "OFF".to_owned(),
            VariableType::Integer if !value.is_empty() => {
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
        }
    }

    pub fn same(&self, new: &str, current: &str) -> Option<String> {
        let new = self.normalize(new);
        let current = self.normalize(current);
        if new == current {
            None
        } else {
            Some(new)
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/mysql_system_vardef.rs"));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get() {
        let v = VariableDefinition::get("sql_mode");
        assert!(v.is_some());

        let v = VariableDefinition::get("sql_mode_2");
        assert!(v.is_none());
    }
}
