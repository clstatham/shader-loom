use naga::{ShaderStage, VectorSize};
use rustc_hash::FxHashMap;

use self::{scope::Scope, value::Value};

pub mod expression;
pub mod scope;
pub mod statement;
pub mod value;

fn type_name(ty: &naga::Type) -> Option<String> {
    match ty.name.as_ref() {
        Some(name) => Some(name.to_owned()),
        None => match ty.inner {
            naga::TypeInner::Scalar { kind, width } => Some(format!("{:?}{}", kind, width * 8)),
            naga::TypeInner::Vector { size, kind, width } => {
                let size = match size {
                    VectorSize::Bi => 2,
                    VectorSize::Tri => 3,
                    VectorSize::Quad => 4,
                };
                let width = width * 8;
                Some(format!("vec{}<{:?}{}>", size, kind, width))
            }
            _ => todo!("{:?}", ty),
        },
    }
}

pub struct Interpreter<'a> {
    shader_stage: ShaderStage,
    verbosity: u8,

    scopes: Vec<Scope<'a>>,
}

impl<'a> Interpreter<'a> {
    pub fn new(shader_stage: ShaderStage, verbosity: u8) -> Self {
        Self {
            shader_stage,
            verbosity,
            scopes: vec![],
        }
    }

    pub fn run(&mut self, module: &'a naga::Module) -> anyhow::Result<()> {
        let entry_point = module
            .entry_points
            .iter()
            .find(|e| e.stage == self.shader_stage)
            .ok_or(anyhow::anyhow!(
                "No entry point found for stage {:?}",
                self.shader_stage
            ))?;

        println!("Entry point: {}", entry_point.name);
        println!("Stage: {:?}", entry_point.stage);

        let body = entry_point.function.body.as_ref().to_owned();
        let args = entry_point
            .function
            .arguments
            .iter()
            .map(|arg| {
                let name = arg
                    .name
                    .as_ref()
                    .ok_or(anyhow::anyhow!("Argument has no name"))?;

                let ty = module
                    .types
                    .get_handle(arg.ty)
                    .map_err(|_| anyhow::anyhow!("Invalid type handle: {:?}", arg.ty))?;

                let ty_name = type_name(ty).ok_or(anyhow::anyhow!("Invalid type: {:?}", ty))?;

                let size = ty.inner.size(module.to_ctx());

                let value = {
                    println!("Enter value for argument `{}` ({}):", name, ty_name);
                    let mut value = String::default();
                    std::io::stdin().read_line(&mut value)?;
                    let value = value.trim();
                    match ty.inner {
                        naga::TypeInner::Scalar { kind, width } => match kind {
                            naga::ScalarKind::Sint => match width {
                                4 => Value::from_pod(&ty.inner, value.parse::<i32>()?),
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Uint => match width {
                                4 => Value::from_pod(&ty.inner, value.parse::<u32>()?),
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Float => match width {
                                4 => Value::from_pod(&ty.inner, value.parse::<f32>()?),
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Bool => {
                                Value::from_pod(&ty.inner, value.parse::<bool>()? as u8)
                            }
                        },
                        naga::TypeInner::Vector { size, kind, width } => {
                            let size = size as usize;
                            let width = width as usize;
                            let mut data = vec![0; size * width];
                            for (i, value) in value.split(',').enumerate() {
                                match kind {
                                    naga::ScalarKind::Sint => match width {
                                        4 => {
                                            let value = value.trim().parse::<i32>()?;
                                            data[i * width..(i + 1) * width]
                                                .copy_from_slice(bytemuck::bytes_of(&value));
                                        }
                                        _ => todo!("{:?}", width),
                                    },
                                    naga::ScalarKind::Uint => match width {
                                        4 => {
                                            let value = value.trim().parse::<u32>()?;
                                            data[i * width..(i + 1) * width]
                                                .copy_from_slice(bytemuck::bytes_of(&value));
                                        }
                                        _ => todo!("{:?}", width),
                                    },
                                    naga::ScalarKind::Float => match width {
                                        4 => {
                                            let value = value.trim().parse::<f32>()?;
                                            data[i * width..(i + 1) * width]
                                                .copy_from_slice(bytemuck::bytes_of(&value));
                                        }
                                        _ => todo!("{:?}", width),
                                    },
                                    naga::ScalarKind::Bool => {
                                        let value = value.trim().parse::<bool>()?;
                                        data[i * width..(i + 1) * width]
                                            .copy_from_slice(bytemuck::bytes_of(&(value as u8)));
                                    }
                                }
                            }
                            Value::from_data(&ty.inner, data)
                        }
                        _ => todo!("{:?}", ty.inner),
                    }
                };

                if size as usize != value.data.len() {
                    return Err(anyhow::anyhow!(
                        "Invalid argument size: expected {}, got {}",
                        size,
                        value.data.len()
                    ));
                }

                Ok((name.to_owned(), value))
            })
            .collect::<anyhow::Result<FxHashMap<_, _>>>()?;

        let mut value = None;
        self.push_scope(args);
        for stmt in body {
            value = self.statement(module, stmt, &entry_point.function)?;
        }
        self.pop_scope();

        if let Some(value) = value {
            println!("Result: {}", value.try_display()?);
        }

        Ok(())
    }

    fn push_scope(&mut self, mut add_values: FxHashMap<String, Value<'a>>) {
        // inherit values from previous scope
        if let Some(scope) = self.scopes.last() {
            for (name, value) in scope.variables.iter() {
                add_values.insert(name.to_owned(), value.to_owned());
            }
        }
        self.scopes.push(Scope {
            variables: add_values,
        });
    }

    fn pop_scope(&mut self) -> Option<Scope<'a>> {
        self.scopes.pop()
    }

    fn current_scope(&mut self) -> Option<&mut Scope<'a>> {
        self.scopes.last_mut()
    }
}
