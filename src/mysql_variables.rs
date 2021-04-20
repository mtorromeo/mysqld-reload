pub struct VariableDefinition {
    pub name: &'static str,
    pub vartype: VariableType,
}

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

include!(concat!(env!("OUT_DIR"), "/mysql_system_vardef.rs"));
