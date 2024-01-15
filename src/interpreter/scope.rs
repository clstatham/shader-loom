use rustc_hash::FxHashMap;

use super::value::Value;

#[derive(Clone, Debug)]
pub struct Scope<'a> {
    pub variables: FxHashMap<String, Value<'a>>,
}

impl<'a> Scope<'a> {
    pub fn try_display(&self, pad: usize) -> anyhow::Result<String> {
        let mut result = String::default();
        for (name, value) in self.variables.iter() {
            result += &format!("{}{}: {}\n", " ".repeat(pad), name, value.try_display()?);
        }
        Ok(result)
    }
}
