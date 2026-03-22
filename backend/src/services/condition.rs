use loco_rs::Result;
use std::collections::HashMap;

pub trait ConditionEvaluator: Send + Sync {
    fn evaluate(&self, condition: &str, vars: &HashMap<String, f64>) -> Result<bool>;
}

pub struct LuaEvaluator;

impl ConditionEvaluator for LuaEvaluator {
    fn evaluate(&self, condition: &str, vars: &HashMap<String, f64>) -> Result<bool> {
        use mlua::{Lua, LuaOptions, StdLib, Value};

        let lua = Lua::new_with(StdLib::MATH | StdLib::STRING, LuaOptions::default())
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

        let globals = lua.globals();
        for (key, value) in vars {
            globals
                .set(key.as_str(), *value)
                .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
        }

        let result: Value = lua
            .load(format!("return ({condition})"))
            .eval()
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

        match result {
            Value::Boolean(b) => Ok(b),
            Value::Integer(n) => Ok(n != 0),
            Value::Number(n) => Ok(n != 0.0),
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn simple_less_than() {
        let eval = LuaEvaluator;
        assert!(eval.evaluate("price < 129", &vars(&[("price", 125.0)])).unwrap());
        assert!(!eval.evaluate("price < 129", &vars(&[("price", 130.0)])).unwrap());
    }

    #[test]
    fn compound_condition() {
        let eval = LuaEvaluator;
        let v = vars(&[("price", 125.0), ("volume", 5000.0)]);
        assert!(eval.evaluate("price < 130 and volume > 1000", &v).unwrap());
        assert!(!eval.evaluate("price < 130 and volume > 10000", &v).unwrap());
    }

    #[test]
    fn math_expressions() {
        let eval = LuaEvaluator;
        let v = vars(&[("price", 100.0), ("sma", 105.0)]);
        assert!(eval.evaluate("price < sma * 0.98", &v).unwrap());
        assert!(!eval.evaluate("price > sma * 1.02", &v).unwrap());
    }

    #[test]
    fn or_condition() {
        let eval = LuaEvaluator;
        let v = vars(&[("price", 200.0), ("rsi", 25.0)]);
        assert!(eval.evaluate("price > 150 or rsi < 30", &v).unwrap());
    }

    #[test]
    fn invalid_lua_returns_error() {
        let eval = LuaEvaluator;
        assert!(eval.evaluate("??? invalid", &HashMap::new()).is_err());
    }
}
