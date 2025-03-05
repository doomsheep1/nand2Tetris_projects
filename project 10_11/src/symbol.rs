use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum Kind {
    STATIC,
    ARG,
    LOCAL,
    THIS,
    NONE,
}

pub struct SymbolTable {
    table: HashMap<String, (Kind, String, usize)>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn define(
        &mut self,
        symbol_name: String,
        symbol_type: String,
        symbol_kind: Kind,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if symbol_name.is_empty() || symbol_kind == Kind::NONE || symbol_type.is_empty() {
            Err(format!("Identifier definition failed, table fields are invalid: {symbol_name},{symbol_type},{:?}",symbol_kind))?;
        }

        if !self.table.keys().any(|key| *key == symbol_name) {
            let var_count = self.var_count(&symbol_kind);
            self.table
                .insert(symbol_name, (symbol_kind, symbol_type, var_count));
        } else {
            Err(format!(
                "Identifier definition failed, {symbol_name} identifier already exists"
            ))?;
        }

        Ok(())
    }

    pub fn var_count(&self, symbol_kind: &Kind) -> usize {
        self.table
            .iter()
            .filter(|&(_curr_name, (curr_kind, _curr_type, _curr_index))| curr_kind == symbol_kind)
            .count()
    }

    pub fn kind_of(&self, symbol_name: &str) -> Option<Kind> {
        if let Some(symbol) = self.table.get(symbol_name) {
            return Some(symbol.0.clone());
        }

        None
    }

    pub fn type_of(&self, symbol_name: &str) -> Option<String> {
        if let Some(symbol) = self.table.get(symbol_name) {
            return Some(symbol.1.to_string());
        }

        None
    }

    pub fn index_of(&self, symbol_name: &str) -> Option<usize> {
        if let Some(symbol) = self.table.get(symbol_name) {
            return Some(symbol.2);
        }

        None
    }

    pub fn reset(&mut self) {
        self.table.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_symbol_definition() {
        let mut dummy_table = SymbolTable::new();
        let identifier_1 = dummy_table.define(
            "test_symbol".to_string(),
            "test_type".to_string(),
            Kind::LOCAL,
        );
        let dup_identifier_1 = dummy_table.define(
            "test_symbol".to_string(),
            "test_type".to_string(),
            Kind::LOCAL,
        );
        let null_definitions = dummy_table.define("".to_string(), "".to_string(), Kind::NONE);
        assert!(identifier_1.is_ok());
        assert!(dup_identifier_1.is_err());
        assert!(null_definitions.is_err());
    }

    #[test]
    fn validate_var_count() {
        let mut dummy_table = SymbolTable::new();
        let var_local_1 = dummy_table.define(
            "test_symbol_1".to_string(),
            "test_type".to_string(),
            Kind::LOCAL,
        );
        let var_local_2 = dummy_table.define(
            "test_symbol_2".to_string(),
            "test_type".to_string(),
            Kind::LOCAL,
        );
        let var_local_3 = dummy_table.define(
            "test_symbol_3".to_string(),
            "test_type".to_string(),
            Kind::LOCAL,
        );
        let var_arg_1 = dummy_table.define(
            "test_symbol_4".to_string(),
            "test_type".to_string(),
            Kind::ARG,
        );
        let var_arg_2 = dummy_table.define(
            "test_symbol_5".to_string(),
            "test_type".to_string(),
            Kind::ARG,
        );
        let local_count = dummy_table.var_count(&Kind::LOCAL);
        let arg_count = dummy_table.var_count(&Kind::ARG);
        assert!(var_local_1.is_ok());
        assert!(var_local_2.is_ok());
        assert!(var_local_3.is_ok());
        assert!(var_arg_1.is_ok());
        assert!(var_arg_2.is_ok());
        assert_eq!(local_count, 3);
        assert_eq!(arg_count, 2);
    }

    #[test]
    fn validate_kind_of() {
        let mut dummy_table = SymbolTable::new();
        let symbol_1 = "test_symbol_1".to_string();
        let symbol_2 = "test_symbol_2".to_string();
        let symbol_3 = "test_symbol_3".to_string();

        let var_1 = dummy_table.define(symbol_1.clone(), "test_type".to_string(), Kind::LOCAL);
        let var_2 = dummy_table.define(symbol_2.clone(), "test_type".to_string(), Kind::ARG);
        let var_3 = dummy_table.define(symbol_3.clone(), "test_type".to_string(), Kind::STATIC);
        let var_1_kind = dummy_table.kind_of(&symbol_1);
        let var_2_kind = dummy_table.kind_of(&symbol_2);
        let var_3_kind = dummy_table.kind_of(&symbol_3);

        assert!(var_1.is_ok());
        assert!(var_2.is_ok());
        assert!(var_3.is_ok());
        assert!(var_1_kind.is_some_and(|kind_val| kind_val == Kind::LOCAL));
        assert!(var_2_kind.is_some_and(|kind_val| kind_val == Kind::ARG));
        assert!(var_3_kind.is_some_and(|kind_val| kind_val == Kind::STATIC));
    }

    #[test]
    fn validate_type_of() {
        let mut dummy_table = SymbolTable::new();
        let type_1 = "test_type_1".to_string();
        let type_2 = "test_type_2".to_string();
        let type_3 = "test_type_3".to_string();
        let var_1 = dummy_table.define("test_1".to_string(), type_1.clone(), Kind::LOCAL);
        let var_2 = dummy_table.define("test_2".to_string(), type_2.clone(), Kind::ARG);
        let var_3 = dummy_table.define("test_3".to_string(), type_3.clone(), Kind::STATIC);
        let var_1_type = dummy_table.type_of("test_1");
        let var_2_type = dummy_table.type_of("test_2");
        let var_3_type = dummy_table.type_of("test_3");
        assert!(var_1.is_ok());
        assert!(var_2.is_ok());
        assert!(var_3.is_ok());
        assert!(var_1_type.is_some_and(|type_val| type_val == type_1));
        assert!(var_2_type.is_some_and(|type_val| type_val == type_2));
        assert!(var_3_type.is_some_and(|type_val| type_val == type_3));
    }

    #[test]
    fn validate_index_of() {
        let mut dummy_table = SymbolTable::new();
        let var_1 = dummy_table.define("test_1".to_string(), "test_type".to_string(), Kind::STATIC);
        let var_2 = dummy_table.define("test_2".to_string(), "test_type".to_string(), Kind::STATIC);
        let var_3 = dummy_table.define("test_3".to_string(), "test_type".to_string(), Kind::STATIC);
        let index_0 = dummy_table.index_of("test_1");
        let index_1 = dummy_table.index_of("test_2");
        let index_2 = dummy_table.index_of("test_3");
        assert!(var_1.is_ok());
        assert!(var_2.is_ok());
        assert!(var_3.is_ok());
        assert!(index_0.is_some_and(|index_val| index_val == 0));
        assert!(index_1.is_some_and(|index_val| index_val == 1));
        assert!(index_2.is_some_and(|index_val| index_val == 2));
    }

    #[test]
    fn validate_reset() {
        let mut dummy_table = SymbolTable::new();
        let var_1 = dummy_table.define("test_1".to_string(), "test_type".to_string(), Kind::STATIC);
        let var_2 = dummy_table.define("test_2".to_string(), "test_type".to_string(), Kind::STATIC);
        let var_3 = dummy_table.define("test_3".to_string(), "test_type".to_string(), Kind::STATIC);
        assert!(var_1.is_ok());
        assert!(var_2.is_ok());
        assert!(var_3.is_ok());
        assert!(!dummy_table.table.is_empty());
        dummy_table.reset();
        assert!(dummy_table.table.is_empty());
    }
}
