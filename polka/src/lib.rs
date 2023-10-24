#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::{collections::HashMap, fmt::Display};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Symbol(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::Symbol(sym) => write!(f, "'{}", sym),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Interpreter {
    stack: VecDeque<Value>,
    name_value: HashMap<String, String>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            name_value: HashMap::new(),
            stack: VecDeque::new(),
        }
    }

    pub fn eval(&mut self, expr: &str) {
        let v: Vec<&str> = expr.split_whitespace().collect();
        for val in v {
            let val = val.trim();
            if val.trim().is_empty() {
                continue;
            }
            if val.contains('+') {
                self.operation_res('+')
            } else if val.contains('*') {
                self.operation_res('*');
            } else if val.contains('/') {
                self.operation_res('/');
            } else if val.contains('-') {
                self.operation_res('-');
            } else if val.contains('\'') {
                if val[1..].to_string().len() > 1 {
                    panic!()
                } else {
                    self.stack.push_back(Value::Symbol(val[1..].to_string()))
                }
            } else if val.contains("set") {
                let operand = self.stack.pop_back();
                let value = self.stack.pop_back();
                match operand.clone().unwrap().to_string().parse::<f64>() {
                    Ok(_) => {
                        panic!()
                    }
                    Err(_) => match value.clone().unwrap().to_string().parse::<f64>() {
                        Ok(_) => {
                            self.name_value.insert(
                                operand.clone().unwrap().to_string()[1..].to_string(),
                                value.clone().unwrap().to_string(),
                            );
                        }
                        Err(_) => {
                            self.name_value.insert(
                                operand.clone().unwrap().to_string()[1..].to_string(),
                                value.clone().unwrap().to_string()[1..].to_string(),
                            );
                        }
                    },
                }
            } else if val.contains('$') {
                let val_to_stack = val[1..].to_string();
                let value_str = self.name_value.get(&val_to_stack.clone());
                match value_str.unwrap().to_string().parse::<f64>() {
                    Ok(_) => {
                        self.stack.push_back(Value::Number(
                            value_str.unwrap().to_string().parse().unwrap(),
                        ));
                    }
                    Err(_) => {
                        self.stack
                            .push_back(Value::Symbol(value_str.unwrap().to_string()));
                    }
                }
            } else {
                match val.parse() {
                    Ok(val) => self.stack.push_back(Value::Number(val)),
                    Err(_) => panic!(),
                }
            }
        }
    }

    pub fn stack(&self) -> &[Value] {
        return self.stack.as_slices().0;
    }

    fn operation_res(&mut self, val: char) {
        if self.stack.is_empty() {
            panic!()
        }
        if let Value::Number(num_b) = self.stack.pop_back().unwrap() {
            if self.stack.is_empty() {
                panic!()
            }
            if let Value::Number(num_a) = self.stack.pop_back().unwrap() {
                match val {
                    '+' => self.stack.push_back(Value::Number(num_b + num_a)),
                    '*' => self.stack.push_back(Value::Number(num_b * num_a)),
                    '-' => self.stack.push_back(Value::Number(num_b - num_a)),
                    '/' => self.stack.push_back(Value::Number(num_b / num_a)),
                    _ => panic!(),
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
