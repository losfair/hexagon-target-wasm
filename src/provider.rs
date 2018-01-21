use std::cell::RefCell;
use parity_wasm as wasm;
use parity_wasm::elements::Opcodes as WasmOpcodes;
use parity_wasm::elements::Opcode as WasmOpcode;
use parity_wasm::elements::BlockType as WasmBlockType;
use parity_wasm::elements::ValueType as WasmValueType;
use parity_wasm::elements::Local as WasmLocal;
use parity_wasm::elements::Module as WasmModule;
use parity_wasm::elements::External as WasmExternal;
use parity_wasm::elements::Internal as WasmInternal;
use parity_wasm::elements::ImportEntry as WasmImportEntry;
use parity_wasm::elements::ExportEntry as WasmExportEntry;
use hexagon_vm_core::hybrid::program_context::CommonProgramContext;
use hexagon_vm_core::hybrid::function::Function;
use hexagon_vm_core::hybrid::basic_block::BasicBlock;
use hexagon_vm_core::hybrid::opcode::OpCode;

struct WasmExecutorInfo {
    store_global_fn: u32,
    load_global_fn: u32
}

pub fn compile_function(f: &Function) -> Vec<u8> {
    let mut builder = wasm::builder::module();

    let info = WasmExecutorInfo {
        store_global_fn: 0,
        load_global_fn: 1
    };

    builder = builder
        .with_import(WasmImportEntry::new(
            "env".into(),
            "store_global".into(),
            WasmExternal::Function(0)
        ))
        .with_import(WasmImportEntry::new(
            "env".into(),
            "load_global".into(),
            WasmExternal::Function(0)
        ));

    let mut target_opcodes: Vec<WasmOpcode> = Vec::new();

    // simulate switching
    // loop n + 2
    // block n + 1
    //   block n
    //     block n - 1
    //       ...
    //         block 0
    //           get_local 0 /* state */
    //           br_table [table] n
    //         [code for basic block 0]
    //         return
    //       ...
    //     [code for basic block n - 1]
    //     return
    //   unreachable

    let n_basic_blocks = f.basic_blocks.len();

    target_opcodes.push(WasmOpcode::Loop(WasmBlockType::NoResult));

    for _ in 0..n_basic_blocks + 2 {
        target_opcodes.push(WasmOpcode::Block(WasmBlockType::NoResult));
    }

    target_opcodes.push(WasmOpcode::GetLocal(0)); // the state variable
    target_opcodes.push(WasmOpcode::BrTable(
        (0..n_basic_blocks as u32).collect(),
        n_basic_blocks as u32
    ));
    target_opcodes.push(WasmOpcode::End);

    for i in 1..n_basic_blocks + 1 {
        let branch_out = (n_basic_blocks + 1 - i) as u32;

        compile_basic_block(
            &mut target_opcodes,
            &f.basic_blocks[i - 1],
            branch_out,
            &info
        );
        target_opcodes.push(WasmOpcode::Br(branch_out));
        target_opcodes.push(WasmOpcode::End);
    }

    target_opcodes.push(WasmOpcode::Unreachable);
    target_opcodes.push(WasmOpcode::End);

    // end of loop
    target_opcodes.push(WasmOpcode::End);

    // end of function
    target_opcodes.push(WasmOpcode::End);

    builder = builder
        .function()
            .signature().build()
            .body()
                .with_locals({
                    let mut v: Vec<WasmLocal> = Vec::new();
                    v.push(WasmLocal::new(1, WasmValueType::I32));
                    for _ in 0..16 {
                        v.push(WasmLocal::new(1, WasmValueType::I64));
                    }
                    v
                })
                .with_opcodes(WasmOpcodes::new(target_opcodes))
            .build()
        .build()
        .with_export(WasmExportEntry::new(
            "main".to_string(),
            WasmInternal::Function(2)
        ));

    let module: WasmModule = builder.build();

    wasm::serialize(module).unwrap()
}

fn compile_basic_block(
    opcodes: &mut Vec<WasmOpcode>,
    basic_block: &BasicBlock,
    branch_out: u32,
    info: &WasmExecutorInfo
) {
    for op in &basic_block.opcodes {
        match *op {
            OpCode::Return => opcodes.push(WasmOpcode::Return),
            OpCode::Branch(target) => {
                opcodes.push(WasmOpcode::I32Const(target as i32));
                opcodes.push(WasmOpcode::SetLocal(0))
            },
            OpCode::ConditionalBranch(a, b) => {
                opcodes.push(WasmOpcode::GetLocal(1));
                opcodes.push(WasmOpcode::I32WrapI64);
                opcodes.push(WasmOpcode::SetLocal(0))
            },
            OpCode::SIAdd(a, b) => {
                opcodes.push(WasmOpcode::GetLocal((a + 1) as u32));
                opcodes.push(WasmOpcode::GetLocal((b + 1) as u32));
                opcodes.push(WasmOpcode::I64Add);
                opcodes.push(WasmOpcode::SetLocal(1))
            },
            OpCode::Eq(a, b) => {
                opcodes.push(WasmOpcode::GetLocal((a + 1) as u32));
                opcodes.push(WasmOpcode::GetLocal((b + 1) as u32));
                opcodes.push(WasmOpcode::I64Eq);
                opcodes.push(WasmOpcode::I64ExtendSI32);
                opcodes.push(WasmOpcode::SetLocal(1))
            },
            OpCode::SIConst64(reg, v) => {
                opcodes.push(WasmOpcode::I64Const(v));
                opcodes.push(WasmOpcode::SetLocal((reg + 1) as u32))
            },
            OpCode::LoadGlobal(dst, src) => {
                opcodes.push(WasmOpcode::I32Const(src as i32));
                opcodes.push(WasmOpcode::Call(info.load_global_fn));
                opcodes.push(WasmOpcode::SetLocal((dst + 1) as u32))
            },
            OpCode::StoreGlobal(dst, src) => {
                opcodes.push(WasmOpcode::GetLocal((src + 1) as u32));
                opcodes.push(WasmOpcode::I32Const(dst as i32));
                opcodes.push(WasmOpcode::Call(info.store_global_fn))
            },
            _ => unimplemented!()
        }
    }
}
