use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormValues(Map<String, Value>);

impl FormValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_map(map: Map<String, Value>) -> Self {
        Self(map)
    }

    pub fn into_map(self) -> Map<String, Value> {
        self.0
    }

    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Value) -> Option<Value> {
        self.0.insert(key.into(), value)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_value(self) -> Value {
        Value::Object(self.0)
    }
}
