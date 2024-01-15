use naga::{Expression, Handle, Module, TypeInner};

use super::{type_name, Interpreter, Value};

pub mod binary;

impl<'a> Interpreter<'a> {
    pub(super) fn expression(
        &mut self,
        module: &'a Module,
        expr: Handle<Expression>,
        func: &naga::Function,
    ) -> anyhow::Result<Value<'a>> {
        if self.verbosity > 1 {
            println!("Expression: {:?}", expr);
        }
        let expr = func.expressions[expr].to_owned();
        match expr {
            naga::Expression::Literal(lit) => match lit {
                naga::Literal::Bool(val) => Ok(Value::from_pod(
                    &TypeInner::Scalar {
                        kind: naga::ScalarKind::Bool,
                        width: 1,
                    },
                    val as u8,
                )),
                naga::Literal::I32(val) => Ok(Value::from_pod(
                    &TypeInner::Scalar {
                        kind: naga::ScalarKind::Sint,
                        width: 4,
                    },
                    val,
                )),
                naga::Literal::U32(val) => Ok(Value::from_pod(
                    &TypeInner::Scalar {
                        kind: naga::ScalarKind::Uint,
                        width: 4,
                    },
                    val,
                )),
                naga::Literal::F32(val) => Ok(Value::from_pod(
                    &TypeInner::Scalar {
                        kind: naga::ScalarKind::Float,
                        width: 4,
                    },
                    val,
                )),
                naga::Literal::F64(val) => Ok(Value::from_pod(
                    &TypeInner::Scalar {
                        kind: naga::ScalarKind::Float,
                        width: 8,
                    },
                    val,
                )),
            },
            naga::Expression::FunctionArgument(arg) => {
                let arg = &func.arguments[arg as usize];
                let name = arg
                    .name
                    .as_ref()
                    .ok_or(anyhow::anyhow!("Argument has no name"))?;
                let value = self
                    .current_scope()
                    .ok_or(anyhow::anyhow!("No scope found"))?
                    .variables
                    .get(name)
                    .ok_or(anyhow::anyhow!("Variable not found: {}", name))?
                    .to_owned();
                Ok(value)
            }
            naga::Expression::Compose { ty, components } => {
                let ty = module.types.get_handle(ty).unwrap();
                let ty_name = type_name(ty).unwrap();
                let size = ty.inner.size(module.to_ctx());
                let mut data = vec![0; size as usize];
                for (i, component) in components.iter().enumerate() {
                    let component = self.expression(module, *component, func)?;
                    let component_size = component.data.len();
                    data[i * component_size..(i + 1) * component_size]
                        .copy_from_slice(&component.data);
                }
                let value = Value::from_data(&ty.inner, data);
                Ok(value)
            }
            naga::Expression::Binary { op, left, right } => {
                self.binary(module, func, op, left, right)
            }
            expr => todo!("{:?}", expr),
        }
    }
}
