use naga::{Expression, Handle, ShaderStage, Statement};
use rustc_hash::FxHashMap;

#[derive(Clone, Debug, Default)]
pub struct Value {
    type_name: String,
    data: Vec<u8>,
}

impl Value {
    pub fn from_data(type_name: &str, data: Vec<u8>) -> Self {
        Self {
            type_name: type_name.to_owned(),
            data,
        }
    }

    pub fn from_pod<T: bytemuck::Pod>(type_name: &str, value: T) -> Self {
        Self {
            type_name: type_name.to_owned(),
            data: bytemuck::bytes_of(&value).to_vec(),
        }
    }

    pub fn try_get<T: bytemuck::Pod>(&self) -> anyhow::Result<&T> {
        let value = bytemuck::try_from_bytes(&self.data).map_err(|_| {
            anyhow::anyhow!("Invalid type: expected {:?}", std::any::type_name::<T>())
        })?;
        Ok(value)
    }

    pub fn try_get_mut<T: bytemuck::Pod>(&mut self) -> anyhow::Result<&mut T> {
        let value = bytemuck::try_from_bytes_mut(&mut self.data).map_err(|_| {
            anyhow::anyhow!("Invalid type: expected {:?}", std::any::type_name::<T>())
        })?;
        Ok(value)
    }

    pub fn try_get_offset<T: bytemuck::Pod>(&self, offset: usize) -> anyhow::Result<&T> {
        let value = bytemuck::try_from_bytes(&self.data[offset..std::mem::size_of::<T>()])
            .map_err(|_| {
                anyhow::anyhow!("Invalid type: expected {:?}", std::any::type_name::<T>())
            })?;
        Ok(value)
    }

    pub fn try_get_offset_mut<T: bytemuck::Pod>(
        &mut self,
        offset: usize,
    ) -> anyhow::Result<&mut T> {
        let value = bytemuck::try_from_bytes_mut(&mut self.data[offset..std::mem::size_of::<T>()])
            .map_err(|_| {
                anyhow::anyhow!("Invalid type: expected {:?}", std::any::type_name::<T>())
            })?;
        Ok(value)
    }

    pub fn try_display(&self) -> anyhow::Result<String> {
        match self.type_name.as_str() {
            "Bool" => Ok(self.try_get::<u8>()?.to_string()),
            "Int32" => Ok(self.try_get::<i32>()?.to_string()),
            "Uint32" => Ok(self.try_get::<u32>()?.to_string()),
            "Float32" => Ok(self.try_get::<f32>()?.to_string()),
            "Float64" => Ok(self.try_get::<f64>()?.to_string()),
            _ => Err(anyhow::anyhow!("Invalid type: {:?}", self.type_name)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub variables: FxHashMap<String, Value>,
}

impl Scope {
    pub fn try_display(&self, pad: usize) -> anyhow::Result<String> {
        let mut result = String::default();
        for (name, value) in self.variables.iter() {
            result += &format!("{}{}: {}\n", " ".repeat(pad), name, value.try_display()?);
        }
        Ok(result)
    }
}

pub struct Interpreter {
    shader_stage: ShaderStage,
    verbosity: u8,

    scopes: Vec<Scope>,
}

impl Interpreter {
    pub fn new(shader_stage: ShaderStage, verbosity: u8) -> Self {
        Self {
            shader_stage,
            verbosity,
            scopes: vec![],
        }
    }

    pub fn run(&mut self, module: naga::Module) -> anyhow::Result<()> {
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

                let ty_name = match ty.name.as_ref() {
                    Some(name) => name.to_owned(),
                    None => match ty.inner {
                        naga::TypeInner::Scalar { kind, width } => {
                            format!("{:?}{}", kind, width * 8)
                        }
                        _ => todo!("{:?}", ty),
                    },
                };

                let size = ty.inner.size(module.to_ctx());

                let value = {
                    println!("Enter value for argument `{}` ({}):", name, ty_name);
                    let mut value = String::default();
                    std::io::stdin().read_line(&mut value)?;
                    let value = value.trim();
                    match ty.inner {
                        naga::TypeInner::Scalar { kind, width } => match kind {
                            naga::ScalarKind::Sint => match width {
                                4 => Value::from_pod("Int32", value.parse::<i32>()?),
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Uint => match width {
                                4 => Value::from_pod("Uint32", value.parse::<u32>()?),
                                _ => todo!("{:?}", width),
                            },
                            naga::ScalarKind::Float => match width {
                                4 => Value::from_pod("Float32", value.parse::<f32>()?),
                                _ => todo!("{:?}", width),
                            },
                            _ => todo!("{:?}", kind),
                        },
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
        self.push_scope(args);
        for stmt in body {
            self.statement(stmt, &entry_point.function)?;
        }
        self.pop_scope();

        Ok(())
    }

    fn push_scope(&mut self, mut add_values: FxHashMap<String, Value>) {
        // inherit values from previous scope
        for scope in self.scopes.iter() {
            for (name, value) in scope.variables.iter() {
                add_values.insert(name.to_owned(), value.to_owned());
            }
        }
        self.scopes.push(Scope {
            variables: add_values,
        });
    }

    fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    fn current_scope(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    fn statement(&mut self, stmt: Statement, func: &naga::Function) -> anyhow::Result<()> {
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
                    .map(|expr| self.expression(expr, func))
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(())
            }
            Statement::Block(block) => {
                // todo: scopes
                for stmt in block.into_iter() {
                    self.statement(stmt.to_owned(), func)?;
                }
                Ok(())
            }
            Statement::Return { value } => {
                let value = value.map(|expr| self.expression(expr, func)).transpose()?;
                Ok(())
            }
            stmt => todo!("{:?}", stmt),
        }
    }

    fn expression(
        &mut self,
        expr: Handle<Expression>,
        func: &naga::Function,
    ) -> anyhow::Result<Value> {
        if self.verbosity > 1 {
            println!("Expression: {:?}", expr);
        }
        let expr = func.expressions[expr].to_owned();
        match expr {
            naga::Expression::Literal(lit) => match lit {
                naga::Literal::Bool(val) => Ok(Value::from_pod("Bool", val as u8)),
                naga::Literal::I32(val) => Ok(Value::from_pod("Int32", val)),
                naga::Literal::U32(val) => Ok(Value::from_pod("Uint32", val)),
                naga::Literal::F32(val) => Ok(Value::from_pod("Float32", val)),
                naga::Literal::F64(val) => Ok(Value::from_pod("Float64", val)),
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
            expr => todo!("{:?}", expr),
        }
    }
}
