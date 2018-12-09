use inflections::Inflect;

use clap::{
    Arg,
    App,
    load_yaml,
};

use svd_parser as svd;

use std::io::{Read, Write};
use std::fs::{
    File,
};

use std::path::Path;

mod convert;

use crate::convert::Convert;

#[derive(Debug, PartialEq)]
enum RacrNode {
    Directory{name: String, sub_nodes: Vec<RacrNode>},
    File{name: String, file_content: racr::FileContent},
}

impl RacrNode {
    fn write(&self, target: &Path) {
        match self {
            RacrNode::File{name, file_content} => {
                let mut output_path = target.join(name);
                output_path.set_extension("racr");
                let mut output_file = File::create(output_path).unwrap();
                write!(&mut output_file, "{}", file_content).unwrap();
            },
            RacrNode::Directory{name, sub_nodes} => {
                std::fs::create_dir_all(&target.join(name)).unwrap();
                for node in sub_nodes {
                    node.write(&target.join(name));
                }
            },
        }
    }
}

#[derive(Debug)]
struct ProgramState {
    racr_nodes: Vec<RacrNode>,
}

impl ProgramState {
    fn init() -> Self {
        ProgramState {
            racr_nodes: Vec::new(),
        }
    }

    fn read_svd(&mut self, svd_file: &Path) {
        // TODO: take in a racr path that selects where the definition will be placed in the racr hiarchy

        let mut input_file = File::open(svd_file).unwrap();
        let mut svd_string = String::new();
        input_file.read_to_string(&mut svd_string).unwrap();
        let svd = svd::parse(&svd_string).unwrap();

        let mut content: Vec<racr::Item> = Vec::new();

        // Create the device
        content.push(svd.convert());


        // Create a module for each peripheral
        let mut peripheral_modules = svd.peripherals.iter().map(|peripheral| {
            let mut content: Vec<racr::Item> = Vec::new();

            content.push(peripheral.convert());

            if let Some(ref registers) = peripheral.registers {
                for register in registers {
                    match register {
                        svd::RegisterCluster::Register(register) => {
                            content.push(register.convert());
                        },
                        svd::RegisterCluster::Cluster(_cluster) => {
                            // TODO: Unroll cluster
                        },
                    }
                }
            }

            racr::Item::Mod(racr::Module {
                ident: peripheral.name.clone().to_snake_case().into(),
                content: Some(content),
            })
        }).collect();
        content.append(&mut peripheral_modules);
        
        self.racr_nodes.push(
            RacrNode::Directory{
                name: svd.name.into(),
                sub_nodes: vec![ RacrNode::File{name: String::from("lib"), file_content: racr::FileContent{content} } ]
            }
        );
    }

    fn write_racr(&self, target: &Path) {
        std::fs::create_dir_all(target).unwrap();
        for node in self.racr_nodes.iter() {
            node.write(target);
        }
    }
}

fn main() {
    let cli_yaml = load_yaml!("cli.yml");
    let cli_matches = App::from_yaml(cli_yaml).get_matches();

    let input_filepath = Path::new(cli_matches.value_of("input").unwrap());
    let output_dirpath = Path::new(cli_matches.value_of("output").unwrap());

    let mut state = ProgramState::init();

    state.read_svd(input_filepath);
    state.write_racr(output_dirpath);

}
