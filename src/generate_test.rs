extern crate hexagon_vm_core;
extern crate parity_wasm;

mod provider;

use hexagon_vm_core::hybrid::program::{Program, NativeFunction};
use hexagon_vm_core::hybrid::function::Function;
use hexagon_vm_core::hybrid::basic_block::BasicBlock;
use hexagon_vm_core::hybrid::opcode::OpCode;
use hexagon_vm_core::hybrid::executor::Executor;

use std::fs;
use std::io::Write;

fn main() {
    let output_dir = std::env::args().nth(1).expect("Output directory required");

    let mut program = Program::from_functions(vec! [
        Function::from_basic_blocks(vec! [
            BasicBlock::from_opcodes(vec! [
                { OpCode::SIConst64(1, 42) },
                { OpCode::SIConst64(2, 66) },
                { OpCode::SIAdd(1, 2) },
                { OpCode::StoreGlobal(0, 0) },
                { OpCode::Return }
            ])
        ])
    ]);

    let mut id: usize = 0;

    for f in &program.functions {
        let path = format!("{}/{}.wasm", output_dir, id);

        println!("Writing to {}", path);

        let code = provider::compile_function(f);

        write_file(path.as_str(), &code);

        id += 1;
    }

    let cfg_path = format!("{}/module.cfg", output_dir);
    println!("Writing CFG: {}", cfg_path);
    write_file(cfg_path.as_str(), program.dump().std_serialize().as_slice());
}

fn write_file(path: &str, data: &[u8]) {
    let mut open_options = fs::OpenOptions::new();
        open_options.write(true).create(true);
        let mut file = open_options.open(
            path
        ).unwrap();
        file.write(&data).unwrap();
}
