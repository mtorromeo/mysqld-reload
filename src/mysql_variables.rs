use std::collections::HashSet;

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
            VariableType::Set => {
                let set: HashSet<_> = value.split_terminator(',').map(|s| s.trim()).collect();
                let mut v: Vec<_> = set.into_iter().collect();
                v.sort_unstable();
                v.join(",")
            }
            _ => value
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

    #[test]
    fn test_normalize_integer() {
        let v = VariableDefinition::get("tmp_table_size").unwrap();
        assert!(matches!(v.vartype, VariableType::Integer));
        assert_eq!("16777216", v.normalize("16M"));
    }

    #[test]
    fn test_normalize_bool() {
        let v = VariableDefinition::get("autocommit").unwrap();
        assert!(matches!(v.vartype, VariableType::Boolean));
        assert_eq!("ON", v.normalize("on"));
        assert_eq!("ON", v.normalize("1"));
        assert_eq!("ON", v.normalize("true"));
        assert_eq!("OFF", v.normalize("0"));
        assert_eq!("OFF", v.normalize("x"));
        assert_eq!("OFF", v.normalize("off"));
    }

    #[test]
    fn test_normalize_set() {
        let v = VariableDefinition::get("sql_mode").unwrap();
        assert!(matches!(v.vartype, VariableType::Set));
        assert_eq!("ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION,NO_ZERO_DATE,NO_ZERO_IN_DATE,ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES", v.normalize("ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION"));
        assert_eq!("ONLY_FULL_GROUP_BY", v.normalize("only_full_group_by"));
        assert_eq!("ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES", v.normalize("ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,STRICT_TRANS_TABLES"));
        assert_eq!("ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES", v.normalize("ONLY_FULL_GROUP_BY, STRICT_TRANS_TABLES , STRICT_TRANS_TABLES "));
        assert_eq!("", v.normalize(""));
    }
}
