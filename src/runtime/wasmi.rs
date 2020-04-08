use std::fs;
use std::path::Path;

use wasmi::memory_units::Pages;
use wasmi::{
    Externals, FuncInstance, FuncRef, ImportsBuilder, MemoryDescriptor, MemoryInstance, MemoryRef,
    ModuleImportResolver, RuntimeArgs, RuntimeValue, Signature, Trap, TrapKind, ValueType,
};

use crate::builtins::Builtins;
use crate::error::Error;
use crate::ValueAddr;

use super::{AsBytes, FromBytes, Functions};

const ABORT_FUNC_INDEX: usize = 1;
const BUILTIN0_FUNC_INDEX: usize = 2;
const BUILTIN1_FUNC_INDEX: usize = 3;
const BUILTIN2_FUNC_INDEX: usize = 4;
const BUILTIN3_FUNC_INDEX: usize = 5;
const BUILTIN4_FUNC_INDEX: usize = 6;

#[derive(Clone, Debug)]
struct HostExternals {
    memory: Memory,
    builtins: Builtins,
}

impl HostExternals {
    fn check_signature(&self, index: usize, signature: &Signature) -> bool {
        let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
            ABORT_FUNC_INDEX => (&[ValueType::I32], None),
            BUILTIN0_FUNC_INDEX => (&[ValueType::I32, ValueType::I32], Some(ValueType::I32)),
            BUILTIN1_FUNC_INDEX => (
                &[ValueType::I32, ValueType::I32, ValueType::I32],
                Some(ValueType::I32),
            ),
            BUILTIN2_FUNC_INDEX => (
                &[
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                ],
                Some(ValueType::I32),
            ),
            BUILTIN3_FUNC_INDEX => (
                &[
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                ],
                Some(ValueType::I32),
            ),
            BUILTIN4_FUNC_INDEX => (
                &[
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                    ValueType::I32,
                ],
                Some(ValueType::I32),
            ),
            _ => return false,
        };
        signature.params() == params && signature.return_type() == ret_ty
    }
}

impl ModuleImportResolver for HostExternals {
    fn resolve_memory(
        &self,
        _field_name: &str,
        _descriptor: &MemoryDescriptor,
    ) -> Result<MemoryRef, wasmi::Error> {
        Ok(self.memory.0.clone())
    }

    fn resolve_func(
        &self,
        field_name: &str,
        signature: &Signature,
    ) -> Result<FuncRef, wasmi::Error> {
        let index = match field_name {
            "opa_abort" => ABORT_FUNC_INDEX,
            "opa_builtin0" => BUILTIN0_FUNC_INDEX,
            "opa_builtin1" => BUILTIN1_FUNC_INDEX,
            "opa_builtin2" => BUILTIN2_FUNC_INDEX,
            "opa_builtin3" => BUILTIN3_FUNC_INDEX,
            "opa_builtin4" => BUILTIN4_FUNC_INDEX,
            _ => {
                return Err(wasmi::Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )))
            }
        };

        if !self.check_signature(index, signature) {
            return Err(wasmi::Error::Instantiation(format!(
                "Export {} has a bad signature",
                field_name
            )));
        }

        let f = match field_name {
            "opa_abort" => {
                FuncInstance::alloc_host(Signature::new(&[ValueType::I32][..], None), index)
            }
            "opa_builtin0" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                index,
            ),
            "opa_builtin1" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                index,
            ),
            "opa_builtin2" => FuncInstance::alloc_host(
                Signature::new(
                    &[
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                    ][..],
                    Some(ValueType::I32),
                ),
                index,
            ),
            "opa_builtin3" => FuncInstance::alloc_host(
                Signature::new(
                    &[
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                    ][..],
                    Some(ValueType::I32),
                ),
                index,
            ),
            "opa_builtin4" => FuncInstance::alloc_host(
                Signature::new(
                    &[
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                    ][..],
                    Some(ValueType::I32),
                ),
                index,
            ),
            _ => unreachable!(),
        };
        Ok(f)
    }
}

impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        let result = match index {
            ABORT_FUNC_INDEX => {
                let addr = args.nth_checked(0)?;
                crate::abort(addr);
                None
            }
            BUILTIN0_FUNC_INDEX => {
                let id = args.nth_checked(0)?;
                let ctx: i32 = args.nth_checked(1)?;
                let result = self.builtins.builtin0(id, ctx.into());
                Some(RuntimeValue::I32(result.into()))
            }
            BUILTIN1_FUNC_INDEX => {
                let id = args.nth_checked(0)?;
                let ctx: i32 = args.nth_checked(1)?;
                let arg0: i32 = args.nth_checked(2)?;
                let result = self.builtins.builtin1(id, ctx.into(), arg0.into());
                Some(RuntimeValue::I32(result.into()))
            }
            BUILTIN2_FUNC_INDEX => {
                let id = args.nth_checked(0)?;
                let ctx: i32 = args.nth_checked(1)?;
                let arg0: i32 = args.nth_checked(2)?;
                let arg1: i32 = args.nth_checked(3)?;
                let result = self
                    .builtins
                    .builtin2(id, ctx.into(), arg0.into(), arg1.into());
                Some(RuntimeValue::I32(result.into()))
            }
            BUILTIN3_FUNC_INDEX => {
                let id = args.nth_checked(0)?;
                let ctx: i32 = args.nth_checked(1)?;
                let arg0: i32 = args.nth_checked(2)?;
                let arg1: i32 = args.nth_checked(3)?;
                let arg2: i32 = args.nth_checked(4)?;
                let result =
                    self.builtins
                        .builtin3(id, ctx.into(), arg0.into(), arg1.into(), arg2.into());
                Some(RuntimeValue::I32(result.into()))
            }
            BUILTIN4_FUNC_INDEX => {
                let id = args.nth_checked(0)?;
                let ctx: i32 = args.nth_checked(1)?;
                let arg0: i32 = args.nth_checked(2)?;
                let arg1: i32 = args.nth_checked(3)?;
                let arg2: i32 = args.nth_checked(4)?;
                let arg3: i32 = args.nth_checked(5)?;
                let result = self.builtins.builtin4(
                    id,
                    ctx.into(),
                    arg0.into(),
                    arg1.into(),
                    arg2.into(),
                    arg3.into(),
                );
                Some(RuntimeValue::I32(result.into()))
            }
            _ => return Err(TrapKind::ElemUninitialized.into()),
        };
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub struct Instance {
    memory: Memory,
    functions: Functions,
    externals: HostExternals,
}

impl Instance {
    pub fn new(module: &Module, memory: Memory) -> Result<Self, Error> {
        let builtins = Builtins::default();
        let externals = HostExternals {
            memory: memory.clone(),
            builtins: builtins.clone(),
        };
        let imports = ImportsBuilder::new().with_resolver("env", &externals);
        let instance = wasmi::ModuleInstance::new(&module.0, &imports)
            .map_err(Error::Wasmi)?
            .assert_no_start();
        let fimpl = FunctionsImpl::new(instance, externals.clone())?;
        let functions = Functions::from_impl(fimpl)?;
        let instance = Instance {
            memory,
            functions,
            externals,
        };
        builtins.replace(instance.clone())?;

        Ok(instance)
    }

    pub fn functions(&self) -> &Functions {
        &self.functions
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }
}

#[derive(Clone, Debug)]
pub struct Memory(MemoryRef);

impl Memory {
    pub fn from_module(_module: &Module) -> Self {
        let memory = MemoryInstance::alloc(Pages(5), None).unwrap();
        Memory(memory)
    }

    pub fn get<T: FromBytes>(&self, addr: ValueAddr) -> Result<T, Error> {
        let start = addr.0 as usize;
        let t = self
            .0
            .with_direct_access(|bytes| T::from_bytes(&bytes[start..]))?;
        Ok(t)
    }

    pub fn get_bytes(&self, addr: ValueAddr, len: usize) -> Result<Vec<u8>, Error> {
        let start = addr.0 as u32;
        self.0.get(start, len).map_err(Error::Wasmi)
    }

    pub fn set<T: AsBytes>(&self, addr: ValueAddr, value: &T) -> Result<(), Error> {
        self.0
            .set(addr.0 as u32, value.as_bytes())
            .map_err(Error::Wasmi)
    }
}

pub struct Module(wasmi::Module);

impl Module {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Module, Error> {
        let bytes = fs::read(path).map_err(Error::FileRead)?;
        Self::from_bytes(bytes)
    }

    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Module, Error> {
        let module = wasmi::Module::from_buffer(&bytes).map_err(Error::Wasmi)?;
        Ok(Module(module))
    }
}

#[derive(Debug)]
pub struct FunctionsImpl {
    module_ref: wasmi::ModuleRef,
    externals: HostExternals,
}

impl FunctionsImpl {
    fn new(module_ref: wasmi::ModuleRef, externals: HostExternals) -> Result<Self, Error> {
        let f = FunctionsImpl {
            module_ref,
            externals,
        };
        Ok(f)
    }

    pub fn builtins(&self) -> Result<i32, Error> {
        let args = [];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("builtins", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }

    pub fn opa_eval_ctx_new(&self) -> Result<i32, Error> {
        let args = [];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_eval_ctx_new", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }

    pub fn opa_eval_ctx_set_input(&self, ctx: i32, input: i32) -> Result<(), Error> {
        let args = [RuntimeValue::I32(ctx), RuntimeValue::I32(input)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_eval_ctx_set_input", &args[..], &mut externals)
            .map(drop)
            .map_err(Error::Wasmi)
    }

    pub fn opa_eval_ctx_set_data(&self, ctx: i32, data: i32) -> Result<(), Error> {
        let args = [RuntimeValue::I32(ctx), RuntimeValue::I32(data)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_eval_ctx_set_data", &args[..], &mut externals)
            .map(drop)
            .map_err(Error::Wasmi)
    }

    pub fn eval(&self, ctx: i32) -> Result<(), Error> {
        let args = [RuntimeValue::I32(ctx)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("eval", &args[..], &mut externals)
            .map(drop)
            .map_err(Error::Wasmi)
    }

    pub fn opa_eval_ctx_get_result(&self, ctx: i32) -> Result<i32, Error> {
        let args = [RuntimeValue::I32(ctx)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_eval_ctx_get_result", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }

    pub fn opa_heap_ptr_get(&self) -> Result<i32, Error> {
        let args = [];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_heap_ptr_get", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }

    pub fn opa_heap_ptr_set(&self, addr: i32) -> Result<(), Error> {
        let args = [RuntimeValue::I32(addr)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_heap_ptr_set", &args[..], &mut externals)
            .map(drop)
            .map_err(Error::Wasmi)
    }

    pub fn opa_heap_top_get(&self) -> Result<i32, Error> {
        let args = [];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_heap_top_get", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }

    pub fn opa_heap_top_set(&self, addr: i32) -> Result<(), Error> {
        let args = [RuntimeValue::I32(addr)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_heap_top_set", &args[..], &mut externals)
            .map(drop)
            .map_err(Error::Wasmi)
    }

    pub fn opa_malloc(&self, len: i32) -> Result<i32, Error> {
        let args = [RuntimeValue::I32(len)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_malloc", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }

    pub fn opa_json_parse(&self, addr: i32, len: i32) -> Result<i32, Error> {
        let args = [RuntimeValue::I32(addr), RuntimeValue::I32(len)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_json_parse", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }

    pub fn opa_json_dump(&self, addr: i32) -> Result<i32, Error> {
        let args = [RuntimeValue::I32(addr)];
        let mut externals = self.externals.clone();
        self.module_ref
            .invoke_export("opa_json_dump", &args[..], &mut externals)
            .map(|v| v.and_then(|r| r.try_into::<i32>()))
            .map_err(Error::Wasmi)
            .transpose()
            .unwrap_or_else(|| Err(Error::InvalidResult("i32")))
    }
}
