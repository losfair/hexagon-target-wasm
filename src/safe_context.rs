use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::Cell;
use std::ops::Deref;
use hexagon_vm_core::hybrid::program_context::CommonProgramContext;

pub struct ContextOwner<'a> {
    ctx: *const CommonProgramContext,
    ref_info: Rc<()>,
    _lt: PhantomData<&'a CommonProgramContext>
}

pub struct ContextHandle {
    ctx: *const CommonProgramContext,
    _ref_info: Rc<()>
}

impl<'a> ContextOwner<'a> {
    pub fn new(inner: &'a CommonProgramContext) -> ContextOwner<'a> {
        ContextOwner {
            ctx: unsafe {
                ::std::mem::transmute::<&'a CommonProgramContext, *const CommonProgramContext>(inner)
            },
            ref_info: Rc::new(()),
            _lt: PhantomData
        }
    }

    pub fn handle(&self) -> ContextHandle {
        ContextHandle {
            ctx: self.ctx,
            _ref_info: self.ref_info.clone()
        }
    }
}

impl<'a> Drop for ContextOwner<'a> {
    fn drop(&mut self) {
        if Rc::strong_count(&self.ref_info) != 1 {
            eprintln!("Dropping owner before all handles are dropped");
            ::std::process::abort();
        }
    }
}

impl ContextHandle {
    pub fn get(&self) -> &CommonProgramContext {
        unsafe {
            &*self.ctx
        }
    }
}

impl Clone for ContextHandle {
    fn clone(&self) -> Self {
        ContextHandle {
            ctx: self.ctx,
            _ref_info: self._ref_info.clone()
        }
    }
}
