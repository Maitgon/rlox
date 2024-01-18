use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn insert(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: &String) -> Result<Value, String> {
        match self.values.get(name) {
            Some(value) => Ok(value.clone()),
            None => {
                match &mut self.enclosing {
                    Some(enclosing) => enclosing.get(name),
                    None => Err(format!("Undefined variable '{}'.", name)),
                }
            }
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        self.get(&name)?;
        self.values.insert(name, value);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}