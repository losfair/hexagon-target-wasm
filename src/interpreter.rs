use std::sync::Arc;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use safe_context::{ContextOwner, ContextHandle};
use provider::WasmExecutor;
use parity_wasm as wasm;
use parity_wasm::elements::ValueType as WasmValueType;
use parity_wasm::elements::Module as WasmModule;
use parity_wasm::interpreter::{CallerContext, RuntimeValue, ExecutionParams};
use parity_wasm::interpreter::{UserDefinedElements, UserFunctionDescriptor, UserFunctionExecutor};
use parity_wasm::elements::Deserialize;
use hexagon_vm_core::hybrid::program_context::CommonProgramContext;

pub struct Interpreter {
    modules: RefCell<Vec<Arc<wasm::ModuleInstanceInterface>>>,
    bridge_executor: BridgeExecutor
}

#[derive(Clone)]
pub struct BridgeExecutor {
    context: Rc<RefCell<Vec<ContextHandle>>>
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            modules: RefCell::new(Vec::new()),
            bridge_executor: BridgeExecutor {
                context: Rc::new(RefCell::new(Vec::new()))
            }
        }
    }
}

impl WasmExecutor for Interpreter {
    fn load_module(&self, mut code: &[u8]) -> Result<usize, ()> {
        let instance = wasm::ProgramInstance::new();
        eprintln!("{}", code.len());
        let module: WasmModule = WasmModule::deserialize(&mut code).unwrap();

        let module = instance.add_module("main", module, None).unwrap();
        let module = wasm::interpreter::native_module(
            module,
            UserDefinedElements {
                globals: HashMap::new(),
                functions: ::std::borrow::Cow::from(vec! [
                    UserFunctionDescriptor::Static(
                        "load_global",
                        &[
                            WasmValueType::I32
                        ],
                        Some(WasmValueType::I64)
                    ),
                    UserFunctionDescriptor::Static(
                        "store_global",
                        &[
                            WasmValueType::I32,
                            WasmValueType::I64
                        ],
                        None
                    )
                ]),
                executor: Some(self.bridge_executor.clone())
            }
        ).unwrap();
        let mut modules = self.modules.borrow_mut();
        modules.push(module);
        Ok(modules.len() - 1)
    }

    fn execute_module(&self, id: usize, ctx: &CommonProgramContext) {
        let m: Arc<wasm::ModuleInstanceInterface> = self.modules.borrow()[id].clone();
        let context_owner = ContextOwner::new(ctx);
        let handle = context_owner.handle();
        self.bridge_executor.context.borrow_mut().push(handle.clone());

        m.execute_export("main", ExecutionParams::default()).unwrap();

        self.bridge_executor.context.borrow_mut().pop();
    }
}

impl UserFunctionExecutor for BridgeExecutor {
    fn execute(&mut self, name: &str, context: CallerContext) -> Result<Option<RuntimeValue>, wasm::interpreter::Error> {
        println!("{}", name);
        unimplemented!()
    }
}
