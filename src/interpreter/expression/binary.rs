use naga::{BinaryOperator, Expression, Function, Handle, Module, TypeInner};

use crate::interpreter::{value::Value, Interpreter};

macro_rules! binary {
    ($result:expr; i32; $op:expr, $left:expr, $right:expr) => {
        match $op {
            BinaryOperator::Add => $result = $left + $right,
            BinaryOperator::Subtract => $result = $left - $right,
            BinaryOperator::Multiply => $result = $left * $right,
            BinaryOperator::Divide => $result = $left / $right,
            BinaryOperator::Modulo => $result = $left % $right,
            BinaryOperator::And => $result = $left & $right,
            BinaryOperator::ExclusiveOr => $result = $left ^ $right,
            BinaryOperator::InclusiveOr => $result = $left | $right,
            _ => todo!("{:?}", $op),
        }
    };
    ($result:expr; u32; $op:expr, $left:expr, $right:expr) => {
        binary!($result; i32; $op, $left, $right)
    };
    ($result:expr; f32; $op:expr, $left:expr, $right:expr) => {
        match $op {
            BinaryOperator::Add => $result = $left + $right,
            BinaryOperator::Subtract => $result = $left - $right,
            BinaryOperator::Multiply => $result = $left * $right,
            BinaryOperator::Divide => $result = $left / $right,
            BinaryOperator::Modulo => $result = $left % $right,
            _ => todo!("{:?}", $op),
        }
    };
    ($result:expr; f64; $op:expr, $left:expr, $right:expr) => {
        binary!($result; f32; $op, $left, $right)
    };
    ($result:expr; bool; $op:expr, $left:expr, $right:expr) => {
        match $op {
            BinaryOperator::And => $result = $left & $right,
            BinaryOperator::ExclusiveOr => $result = $left ^ $right,
            BinaryOperator::InclusiveOr => $result = $left | $right,
            BinaryOperator::Equal => $result = ($left == $right) as u8,
            BinaryOperator::NotEqual => $result = ($left != $right) as u8,
            _ => todo!("{:?}", $op),
        }
    };

}

impl<'a> Interpreter<'a> {
    pub(super) fn binary(
        &mut self,
        module: &'a Module,
        func: &Function,
        op: naga::BinaryOperator,
        left: Handle<Expression>,
        right: Handle<Expression>,
    ) -> anyhow::Result<Value<'a>> {
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
        let mut result = Value::from_data(left.ty, vec![0; size]);

        match (left.ty, right.ty) {
            (
                TypeInner::Scalar {
                    kind: left_kind,
                    width: left_width,
                },
                TypeInner::Scalar {
                    kind: right_kind,
                    width: right_width,
                },
            ) => {
                if left_kind != right_kind {
                    return Err(anyhow::anyhow!(
                        "Invalid binary expression: left kind {:?}, right kind {:?}",
                        left_kind,
                        right_kind
                    ));
                }
                if left_width != right_width {
                    return Err(anyhow::anyhow!(
                        "Invalid binary expression: left width {}, right width {}",
                        left_width,
                        right_width
                    ));
                }

                match left_kind {
                    naga::ScalarKind::Sint => match left_width {
                        4 => {
                            let left = left.try_get::<i32>()?;
                            let right = right.try_get::<i32>()?;
                            let r = result.try_get_mut::<i32>()?;
                            binary!(*r; i32; op, *left, *right);
                        }
                        _ => todo!("{:?}", left_width),
                    },
                    naga::ScalarKind::Uint => match left_width {
                        4 => {
                            let left = left.try_get::<u32>()?;
                            let right = right.try_get::<u32>()?;
                            let r = result.try_get_mut::<u32>()?;
                            binary!(*r; u32; op, *left, *right);
                        }
                        _ => todo!("{:?}", left_width),
                    },
                    naga::ScalarKind::Float => match left_width {
                        4 => {
                            let left = left.try_get::<f32>()?;
                            let right = right.try_get::<f32>()?;
                            let r = result.try_get_mut::<f32>()?;
                            binary!(*r; f32; op, *left, *right);
                        }
                        8 => {
                            let left = left.try_get::<f64>()?;
                            let right = right.try_get::<f64>()?;
                            let r = result.try_get_mut::<f64>()?;
                            binary!(*r; f64; op, *left, *right);
                        }
                        _ => todo!("{:?}", left_width),
                    },
                    naga::ScalarKind::Bool => match left_width {
                        1 => {
                            let left = left.try_get::<u8>()?;
                            let right = right.try_get::<u8>()?;
                            let r = result.try_get_mut::<u8>()?;
                            binary!(*r; bool; op, *left, *right);
                        }
                        _ => todo!("{:?}", left_width),
                    },
                }
            }
            (
                &TypeInner::Vector {
                    size: left_size,
                    kind: left_kind,
                    width: left_width,
                },
                &TypeInner::Vector {
                    size: right_size,
                    kind: right_kind,
                    width: right_width,
                },
            ) => {
                if left_size != right_size {
                    return Err(anyhow::anyhow!(
                        "Invalid binary expression: left size {:?}, right size {:?}",
                        left_size,
                        right_size
                    ));
                }
                if left_kind != right_kind {
                    return Err(anyhow::anyhow!(
                        "Invalid binary expression: left kind {:?}, right kind {:?}",
                        left_kind,
                        right_kind
                    ));
                }
                if left_width != right_width {
                    return Err(anyhow::anyhow!(
                        "Invalid binary expression: left width {}, right width {}",
                        left_width,
                        right_width
                    ));
                }

                match left_kind {
                    naga::ScalarKind::Sint => match left_width {
                        4 => {
                            for i in 0..left_size as usize {
                                let left = left.try_get_offset::<i32>(i * 4)?;
                                let right = right.try_get_offset::<i32>(i * 4)?;
                                let r = result.try_get_offset_mut::<i32>(i * 4)?;
                                binary!(*r; i32; op, *left, *right);
                            }
                        }
                        _ => todo!("{:?}", left_width),
                    },
                    naga::ScalarKind::Uint => match left_width {
                        4 => {
                            for i in 0..left_size as usize {
                                let left = left.try_get_offset::<u32>(i * 4)?;
                                let right = right.try_get_offset::<u32>(i * 4)?;
                                let r = result.try_get_offset_mut::<u32>(i * 4)?;
                                binary!(*r; u32; op, *left, *right);
                            }
                        }
                        _ => todo!("{:?}", left_width),
                    },
                    naga::ScalarKind::Float => match left_width {
                        4 => {
                            for i in 0..left_size as usize {
                                let left = left.try_get_offset::<f32>(i * 4)?;
                                let right = right.try_get_offset::<f32>(i * 4)?;
                                let r = result.try_get_offset_mut::<f32>(i * 4)?;
                                binary!(*r; f32; op, *left, *right);
                            }
                        }
                        8 => {
                            for i in 0..left_size as usize {
                                let left = left.try_get_offset::<f64>(i * 8)?;
                                let right = right.try_get_offset::<f64>(i * 8)?;
                                let r = result.try_get_offset_mut::<f64>(i * 8)?;
                                binary!(*r; f64; op, *left, *right);
                            }
                        }
                        _ => todo!("{:?}", left_width),
                    },
                    naga::ScalarKind::Bool => match left_width {
                        1 => {
                            for i in 0..left_size as usize {
                                let left = left.try_get_offset::<u8>(i)?;
                                let right = right.try_get_offset::<u8>(i)?;
                                let r = result.try_get_offset_mut::<u8>(i)?;
                                binary!(*r; bool; op, *left, *right);
                            }
                        }
                        _ => todo!("{:?}", left_width),
                    },
                }
            }
            _ => todo!("{:?} {:?}", left.ty, right.ty),
        }

        Ok(result)
    }
}
