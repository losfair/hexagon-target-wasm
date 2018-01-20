use hexagon_vm_core::hybrid::program::{Program, NativeFunction};
use hexagon_vm_core::hybrid::function::Function;
use hexagon_vm_core::hybrid::basic_block::BasicBlock;
use hexagon_vm_core::hybrid::opcode::OpCode;
use hexagon_vm_core::hybrid::executor::Executor;
use hexagon_vm_core::hybrid::program_context::ProgramContext;
use provider::WasmJitProvider;
use interpreter::Interpreter;

#[test]
fn test_basic() {
    let mut program = Program::from_functions(vec! [
        Function::from_basic_blocks(vec! [
            BasicBlock::from_opcodes(vec! [
                { OpCode::Return }
            ])
        ])
    ]);
    let executor = Executor::new();
    let jit_provider = WasmJitProvider::new(Interpreter::new());
    executor.eval_program(&ProgramContext::new(&executor, program, Some(jit_provider)), 0);

    assert_eq!(executor.read_global(0), 42);
}
