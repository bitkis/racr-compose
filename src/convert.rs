use inflections::Inflect;

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
                description: None,
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
            let racr_registers = svd_registers.iter().map(|x| {
                let reg_info = match x.clone().left().unwrap() {
                    svd_parser::Register::Single(info) => info,
                    svd_parser::Register::Array(info, _) => info,
                };


                racr::RegisterSlot::Single {
                    instance: racr::RegisterInstance{
                        ident: reg_info.name.clone().to_snake_case().into(),
                        ty: racr::RegisterType::Single{path: racr::Ident::from(reg_info.name.clone().to_pascal_case()).into()},
                    },
                    offset: reg_info.address_offset as usize,
                }
            }).collect();

            content.push(
                racr::PeripheralDefinition {
                    ident: self.name.clone().to_pascal_case().into(),
                    description: self.description.clone(),
                    registers: racr_registers,
                }.into()
            );

            content.append(&mut svd_registers.iter().map(|x| {
                let reg_info = match x.clone().left().unwrap() {
                    svd_parser::Register::Single(info) => info,
                    svd_parser::Register::Array(info, _) => info,
                };

                let fields = if let Some(fields) = reg_info.fields {
                    fields.iter().map(|x| {
                        racr::FieldInstance {
                            ident: x.name.clone().to_snake_case().into(),
                            description: x.description.clone(),
                            bit_start: x.bit_range.offset as usize,
                            bit_end: (x.bit_range.offset + x.bit_range.width) as usize,
                            access: Some(x.access.unwrap().convert()),
                        }
                    }).collect()
                } else {
                    Vec::new()
                };

                racr::RegisterDefinition{
                    access: reg_info.access.unwrap().convert(),
                    ident: reg_info.name.clone().to_pascal_case().into(),
                    description: Some(reg_info.description.clone()),
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
