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
    DirBuilder,
};

use std::path::Path;

mod convert;

use crate::convert::Convert;

fn main() {
    let cli_yaml = load_yaml!("cli.yml");
    let cli_matches = App::from_yaml(cli_yaml).get_matches();

    let input_filepath = Path::new(cli_matches.value_of("input").unwrap());
    let output_dirpath = Path::new(cli_matches.value_of("output").unwrap());

    DirBuilder::new().recursive(true).create(output_dirpath).unwrap();

    let mut input_file = File::open(input_filepath).unwrap();
    let mut svd_string = String::new();
    input_file.read_to_string(&mut svd_string).unwrap();
    let svd = svd::parse(&svd_string);

    // Write out the unit/device definition
    let mut output_file_lib = File::create(output_dirpath.join("lib.racr")).unwrap();
    let racr_unit = svd.convert();
    writeln!(&mut output_file_lib, "{}", racr_unit).unwrap();

    // Write out peripheral definitions
    for peripheral in svd.peripherals {
        if let Some(racr_peripheral) = peripheral.convert() {
            let mut output_file = File::create(output_dirpath.join(String::from(peripheral.name.clone().to_snake_case()) + ".racr")).unwrap();
            writeln!(&mut output_file, "{}", racr_peripheral).unwrap();
        }
    }



    println!("Hello World!");
}
