use naga::{Module, Statement};
use rustc_hash::FxHashMap;

use super::{Interpreter, Value};

impl<'a> Interpreter<'a> {
    pub(super) fn statement(
        &mut self,
        module: &'a Module,
        stmt: Statement,
        func: &naga::Function,
    ) -> anyhow::Result<Option<Value<'a>>> {
        if self.verbosity > 0 {
            println!("Statement: {:?}", stmt);
        }
        if self.verbosity > 0 {
            if let Some(scope) = self.current_scope() {
                println!("Variables:\n{}", scope.try_display(2)?);
            }
        }
        match stmt {
            Statement::Emit(exprs) => {
                exprs
                    .into_iter()
                    .map(|expr| self.expression(module, expr, func))
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(None)
            }
            Statement::Block(block) => {
                self.push_scope(FxHashMap::default());
                let mut value = None;
                for stmt in block.into_iter() {
                    value = self.statement(module, stmt.to_owned(), func)?;
                }
                self.pop_scope();
                Ok(value)
            }
            Statement::Return { value } => {
                let value = value
                    .map(|expr| self.expression(module, expr, func))
                    .transpose()?;
                Ok(value)
            }
            stmt => todo!("{:?}", stmt),
        }
    }
}
