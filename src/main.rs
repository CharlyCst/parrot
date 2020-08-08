use std::path::PathBuf;

mod data;

fn main() {
    println!("{}", data::hello_world());
    data::initialize(PathBuf::from("."));
}
