use naga::TypeInner;

#[derive(Clone, Debug)]
pub struct Value<'a> {
    pub ty: &'a TypeInner,
    pub data: Vec<u8>,
}

impl<'a> Value<'a> {
    pub fn from_data(ty: &'a TypeInner, data: Vec<u8>) -> Self {
        Self { ty, data }
    }

    pub fn from_pod<T: bytemuck::Pod>(ty: &'a TypeInner, value: T) -> Self {
        Self {
            ty,
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
        let value = bytemuck::try_from_bytes(&self.data[offset..offset + std::mem::size_of::<T>()])
            .map_err(|_| {
                anyhow::anyhow!("Invalid type: expected {:?}", std::any::type_name::<T>())
            })?;
        Ok(value)
    }

    pub fn try_get_offset_mut<T: bytemuck::Pod>(
        &mut self,
        offset: usize,
    ) -> anyhow::Result<&mut T> {
        let value =
            bytemuck::try_from_bytes_mut(&mut self.data[offset..offset + std::mem::size_of::<T>()])
                .map_err(|_| {
                    anyhow::anyhow!("Invalid type: expected {:?}", std::any::type_name::<T>())
                })?;
        Ok(value)
    }

    pub fn try_display(&self) -> anyhow::Result<String> {
        match self.ty {
            TypeInner::Scalar { kind, width } => match kind {
                naga::ScalarKind::Sint => match width {
                    4 => Ok(self.try_get::<i32>()?.to_string()),
                    _ => todo!("{:?}", width),
                },
                naga::ScalarKind::Uint => match width {
                    4 => Ok(self.try_get::<u32>()?.to_string()),
                    _ => todo!("{:?}", width),
                },
                naga::ScalarKind::Float => match width {
                    4 => Ok(self.try_get::<f32>()?.to_string()),
                    _ => todo!("{:?}", width),
                },
                naga::ScalarKind::Bool => Ok(self.try_get::<u8>()?.to_string()),
            },
            TypeInner::Vector { size, kind, width } => {
                let size = *size as usize;
                let width = *width as usize;
                let mut result = String::default();
                for i in 0..size {
                    match kind {
                        naga::ScalarKind::Sint => match width {
                            4 => {
                                let value = self.try_get_offset::<i32>(i * width)?;
                                result += &format!("{}, ", value);
                            }
                            _ => todo!("{:?}", width),
                        },
                        naga::ScalarKind::Uint => match width {
                            4 => {
                                let value = self.try_get_offset::<u32>(i * width)?;
                                result += &format!("{}, ", value);
                            }
                            _ => todo!("{:?}", width),
                        },
                        naga::ScalarKind::Float => match width {
                            4 => {
                                let value = self.try_get_offset::<f32>(i * width)?;
                                result += &format!("{}, ", value);
                            }
                            _ => todo!("{:?}", width),
                        },
                        naga::ScalarKind::Bool => {
                            let value = self.try_get_offset::<u8>(i * width)?;
                            result += &format!("{}, ", value);
                        }
                    }
                }
                Ok(format!("[{}]", result.trim_end_matches(", ")))
            }
            _ => todo!("{:?}", self.ty),
        }
    }
}
