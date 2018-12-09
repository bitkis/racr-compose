use inflections::Inflect;
use svd_parser::svd;

pub(crate) trait Convert<T> {
    fn convert(&self) -> T;
}

impl Convert<Vec<racr::Item>> for svd_parser::Device {
    fn convert(&self) -> Vec<racr::Item> {
        let mut content = Vec::new();

        let peripherals = self.peripherals.iter().map(|x| 
            racr::PeripheralInstance{
                ident: x.name.clone().to_snake_case().into(),
                path: racr::Ident::from(x.name.clone().to_pascal_case()).into(),
                address: x.base_address as usize,

            }).collect();

        content.push(
            racr::DeviceDefinition {
                ident: self.name.clone().to_pascal_case().into(),
                documentation: None,
                peripherals,
            }.into());

        content
    }
}

impl Convert<Vec<racr::Item>> for svd_parser::Peripheral {
    fn convert(&self) -> Vec<racr::Item> {
        let mut content = Vec::new();

        if let Some(ref derived_from) = self.derived_from {
            content.push(
                racr::Item::Use( racr::Use {
                    tree: racr::UseTree::Ident(derived_from.clone().into()),
                } )
            )
        } else if let Some(ref svd_registers) = self.registers.clone() {
            // Add peripheral definition
            let racr_registers = svd_registers.iter().fold(Vec::new(), |mut racr_registers, reg| {
                match reg.clone() {
                    svd::RegisterCluster::Register(svd::Register::Single(info)) => {
                        racr_registers.push(racr::RegisterSlot::Single {
                            instance: racr::RegisterInstance{
                                ident: info.name.clone().to_snake_case().into(),
                                ty: racr::RegisterType::Single{path: racr::Ident::from(info.name.clone().to_pascal_case()).into()},
                            },
                            offset: info.address_offset as usize,
                        });
                    },
                    svd::RegisterCluster::Register(svd::Register::Array(info, array)) => {
                        if Some(array.dim_increment*8) == info.size {
                            racr_registers.push(racr::RegisterSlot::Single {
                                instance: racr::RegisterInstance{
                                    ident: info.name.clone().to_snake_case().into(),
                                    ty: racr::RegisterType::Array{path: racr::Ident::from(info.name.clone().to_pascal_case()).into(), size: array.dim as usize},
                                },
                                offset: info.address_offset as usize,
                            });
                        } else {
                            // TODO: unroll array into many registers
                            racr_registers.push(racr::RegisterSlot::Single {
                                instance: racr::RegisterInstance{
                                    ident: info.name.clone().to_snake_case().into(),
                                    ty: racr::RegisterType::Single{path: racr::Ident::from(info.name.clone().to_pascal_case()).into()},
                                },
                                offset: info.address_offset as usize,
                            });
                        }
                    },
                    svd::RegisterCluster::Cluster(_) => unimplemented!(),
                }
                racr_registers
            });

            content.push(
                racr::PeripheralDefinition {
                    ident: self.name.clone().to_pascal_case().into(),
                    documentation: self.description.clone(),
                    registers: racr_registers,
                }.into()
            );

            // Add register definitions
            content.append(&mut svd_registers.iter().map(|x| {
                let reg_info = match x.clone() {
                    svd::RegisterCluster::Register(svd::Register::Single(info)) => info,
                    svd::RegisterCluster::Register(svd::Register::Array(info, _)) => info,
                    svd::RegisterCluster::Cluster(_) => unimplemented!(),
                };

                let fields = if let Some(fields) = reg_info.fields {
                    fields.iter().map(|x| {
                        racr::FieldInstance {
                            ident: x.name.clone().to_snake_case().into(),
                            documentation: x.description.clone(),
                            bit_range: (x.bit_range.offset as usize)..((x.bit_range.offset + x.bit_range.width) as usize),
                            access: x.access.map(|access| access.convert()),
                            variants: Vec::new(),
                        }
                    }).collect()
                } else {
                    Vec::new()
                };

                racr::RegisterDefinition{
                    access: reg_info.access.map(|access| access.convert()).unwrap_or(racr::Access::ReadWrite),
                    ident: reg_info.name.clone().to_pascal_case().into(),
                    documentation: Some(reg_info.description.clone()),
                    size: reg_info.size.unwrap() as usize,
                    reset_value: reg_info.reset_value.map(|x| x as u128),
                    fields,
                }.into()
                }).collect());

        }

        content
    }
}

impl Convert<racr::Access> for svd_parser::Access {
    fn convert(&self) -> racr::Access {
        match self {
            svd_parser::Access::ReadOnly => racr::Access::ReadOnly,
            svd_parser::Access::ReadWrite => racr::Access::ReadWrite,
            svd_parser::Access::WriteOnly => racr::Access::WriteOnly,
            svd_parser::Access::WriteOnce => racr::Access::WriteOnly,
            svd_parser::Access::ReadWriteOnce => racr::Access::ReadWrite,
        }
    }
}
