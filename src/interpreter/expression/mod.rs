use naga::{Expression, Handle, Module, TypeInner};

use super::{type_name, Interpreter, Value};

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
                let left = self.expression(module, left, func)?;
                let right = self.expression(module, right, func)?;
                let size = left.data.len();
                if size != right.data.len() {
                    return Err(anyhow::anyhow!(
                        "Invalid binary expression: left size {}, right size {}",
                        left.data.len(),
                        right.data.len()
                    ));
                }
                let mut data = vec![0; size];

                match op {
                    naga::BinaryOperator::Add => match left.ty {
                        TypeInner::Scalar { kind, width } => match kind {
                            naga::ScalarKind::Sint => match width {
                                4 => {
                                    let left = left.try_get::<i32>()?;
                                    let right = right.try_get::<i32>()?;
                                    let result = left + right;
                                    data.copy_from_slice(bytemuck::bytes_of(&result));
                                }
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Uint => match width {
                                4 => {
                                    let left = left.try_get::<u32>()?;
                                    let right = right.try_get::<u32>()?;
                                    let result = left + right;
                                    data.copy_from_slice(bytemuck::bytes_of(&result));
                                }
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Float => match width {
                                4 => {
                                    let left = left.try_get::<f32>()?;
                                    let right = right.try_get::<f32>()?;
                                    let result = left + right;
                                    data.copy_from_slice(bytemuck::bytes_of(&result));
                                }
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Bool => {
                                let left = left.try_get::<u8>()?;
                                let right = right.try_get::<u8>()?;
                                let result = left + right;
                                data.copy_from_slice(bytemuck::bytes_of(&result));
                            }
                        },
                        TypeInner::Vector { size, kind, width } => {
                            if let TypeInner::Vector {
                                size: right_size,
                                kind: right_kind,
                                width: right_width,
                            } = right.ty
                            {
                                if size != right_size || kind != right_kind || width != right_width
                                {
                                    return Err(anyhow::anyhow!(
                                        "Invalid binary expression: left type {:?}, right type {:?}",
                                        left.ty,
                                        right.ty
                                    ));
                                }
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Invalid binary expression: left type {:?}, right type {:?}",
                                    left.ty,
                                    right.ty
                                ));
                            }
                            let size = *size as usize;
                            let width = *width as usize;
                            for i in 0..size {
                                match kind {
                                    naga::ScalarKind::Sint => match width {
                                        4 => {
                                            let left = left.try_get_offset::<i32>(i * width)?;
                                            let right = right.try_get_offset::<i32>(i * width)?;
                                            let result = left + right;
                                            data[i * width..(i + 1) * width]
                                                .copy_from_slice(bytemuck::bytes_of(&result));
                                        }
                                        _ => todo!("{:?}", width),
                                    },
                                    naga::ScalarKind::Uint => match width {
                                        4 => {
                                            let left = left.try_get_offset::<u32>(i * width)?;
                                            let right = right.try_get_offset::<u32>(i * width)?;
                                            let result = left + right;
                                            data[i * width..(i + 1) * width]
                                                .copy_from_slice(bytemuck::bytes_of(&result));
                                        }
                                        _ => todo!("{:?}", width),
                                    },
                                    naga::ScalarKind::Float => match width {
                                        4 => {
                                            let left = left.try_get_offset::<f32>(i * width)?;
                                            let right = right.try_get_offset::<f32>(i * width)?;
                                            let result = left + right;
                                            data[i * width..(i + 1) * width]
                                                .copy_from_slice(bytemuck::bytes_of(&result));
                                        }
                                        _ => todo!("{:?}", width),
                                    },
                                    naga::ScalarKind::Bool => {
                                        let left = left.try_get_offset::<u8>(i * width)?;
                                        let right = right.try_get_offset::<u8>(i * width)?;
                                        let result = left + right;
                                        data[i * width..(i + 1) * width]
                                            .copy_from_slice(bytemuck::bytes_of(&result));
                                    }
                                }
                            }
                        }
                        _ => todo!("{:?}", left.ty),
                    },
                    _ => todo!("{:?}", op),
                }

                let value = Value::from_data(left.ty, data);

                Ok(value)
            }
            expr => todo!("{:?}", expr),
        }
    }
}
