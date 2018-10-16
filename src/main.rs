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

fn main() {
    let cli_yaml = load_yaml!("cli.yml");
    let cli_matches = App::from_yaml(cli_yaml).get_matches();

    let input_file = cli_matches.value_of("input").unwrap();
    let output_file = cli_matches.value_of("input").unwrap();

    let mut svd_string = String::new();
    File::open(input_file).unwrap().read_to_string(&mut svd_string).unwrap();
    let svd = svd::parse(&svd_string);

    println!("Hello World!");
}
