use heck::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::mem;
use wit_bindgen_gen_core::wit_parser::abi::{
    AbiVariant, Bindgen, Bitcast, Instruction, LiftLower, WasmType, WitxInstruction,
};
use wit_bindgen_gen_core::{wit_parser::*, Direction, Files, Generator, Ns};

#[derive(Default)]
pub struct TinyGo {
    src: Source,
    in_import: bool,
    opts: Opts,
    funcs: HashMap<String, Vec<Func>>,
    i64_return_pointer_area_size: usize,
    sizes: SizeAlign,
    names: Ns,

    // The set of types that are considered public (aka need to be in the
    // header file) which are anonymous and we're effectively monomorphizing.
    // This is discovered lazily when printing type names.
    public_anonymous_types: BTreeSet<TypeId>,

    // This is similar to `public_anonymous_types` where it's discovered
    // lazily, but the set here are for private types only used in the
    // implementation of functions. These types go in the implementation file,
    // not the header file.
    private_anonymous_types: BTreeSet<TypeId>,

    // Type definitions for the given `TypeId`. This is printed topologically
    // at the end.
    types: HashMap<TypeId, wit_bindgen_gen_core::Source>,

    needs_string: bool,
}

struct Func {
    src: Source,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
pub struct Opts {
    // ...
}

impl Opts {
    pub fn build(&self) -> TinyGo {
        let mut r = TinyGo::new();
        r.opts = self.clone();
        r
    }
}

impl TinyGo {
    pub fn new() -> TinyGo {
        TinyGo::default()
    }

    fn docs(&mut self, docs: &Docs) {
        let docs = match &docs.contents {
            Some(docs) => docs,
            None => return,
        };
        for line in docs.trim().lines() {
            self.src.go("// ");
            self.src.go(line);
            self.src.go("\n");
        }
    }
}

#[derive(Default)]
struct Source {
    src: wit_bindgen_gen_core::Source,
}

impl Generator for TinyGo {
    fn preprocess_one(&mut self, iface: &Interface, dir: Direction) {
        drop((iface, dir));
    }

    fn type_record(
        &mut self,
        iface: &Interface,
        id: TypeId,
        name: &str,
        record: &Record,
        docs: &Docs,
    ) {
        unimplemented!("type_record unimplemented")
    }

    fn type_variant(
        &mut self,
        iface: &Interface,
        id: TypeId,
        name: &str,
        variant: &Variant,
        docs: &Docs,
    ) {
        unimplemented!("type_variant unimplemented")
    }

    fn type_resource(&mut self, iface: &Interface, ty: ResourceId) {
        drop((iface, ty));
    }

    fn type_alias(&mut self, iface: &Interface, id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.docs(docs);
        unimplemented!("type_alias unimplemented")
    }

    fn type_list(&mut self, iface: &Interface, id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        unimplemented!("type_list unimplemented")
    }

    fn type_pointer(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        const_: bool,
        ty: &Type,
        docs: &Docs,
    ) {
        drop((iface, _id, name, const_, ty, docs));
    }

    fn type_builtin(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        drop((iface, _id, name, ty, docs));
    }

    fn type_push_buffer(
        &mut self,
        iface: &Interface,
        id: TypeId,
        name: &str,
        ty: &Type,
        docs: &Docs,
    ) {
        self.type_pull_buffer(iface, id, name, ty, docs);
    }

    fn type_pull_buffer(
        &mut self,
        iface: &Interface,
        id: TypeId,
        name: &str,
        ty: &Type,
        docs: &Docs,
    ) {
        unimplemented!("type_pull buffer unimplemented")

    }

    fn import(&mut self, iface: &Interface, func: &Function) {
        unimplemented!("import unimplemented")
    }

    fn export(&mut self, iface: &Interface, func: &Function) {
        unimplemented!("export unimplemented")
    }

    fn finish_one(&mut self, iface: &Interface, files: &mut Files) {
        unimplemented!("finish_one unimplemented")
    }
}

impl Source {
    fn go(&mut self, s: &str) {
        self.src.push_str(s);
    }
}

fn wasm_type(ty: WasmType) -> &'static str {
    match ty {
        WasmType::I32 => "int",
        WasmType::I64 => "int64",
        WasmType::F32 => "float32",
        WasmType::F64 => "float64",
    }
}
