use heck::*;
use std::mem;
use std::collections::{HashMap};
use wit_bindgen_gen_core::wit_parser::abi::{WasmType};
use wit_bindgen_gen_core::{wit_parser::*, Direction, Files, Generator, Ns};

#[derive(Default)]
pub struct TinyGo {
    src: Source,
    // in_import: bool,
    opts: Opts,
    // funcs: HashMap<String, Vec<Func>>,
    // i64_return_pointer_area_size: usize,
    // sizes: SizeAlign,
    names: Ns,

    // The set of types that are considered public (aka need to be in the
    // header file) which are anonymous and we're effectively monomorphizing.
    // This is discovered lazily when printing type names.
    // public_anonymous_types: BTreeSet<TypeId>,

    // This is similar to `public_anonymous_types` where it's discovered
    // lazily, but the set here are for private types only used in the
    // implementation of functions. These types go in the implementation file,
    // not the header file.
    // private_anonymous_types: BTreeSet<TypeId>,

    // Type definitions for the given `TypeId`. This is printed topologically
    // at the end.
    _types: HashMap<TypeId, wit_bindgen_gen_core::Source>,
}

// struct Func {
//     src: Source,
// }

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

    fn print_package(&mut self, iface: &Interface) {
        let name = iface.name.to_snake_case();
        self.src.go(&format!("package {}\n\n", name));
    }
}

#[derive(Default)]
struct Source {
    src: wit_bindgen_gen_core::Source,
    binding_header: wit_bindgen_gen_core::Source,
}

impl Generator for TinyGo {
    fn preprocess_one(&mut self, iface: &Interface, dir: Direction) {
        self.print_package(iface);
        
        drop(dir);
    }

    fn type_record(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        record: &Record,
        docs: &Docs,
    ) {
        // let prev = mem::take(&mut self.src);
        self.docs(docs);
        if record.is_flags() {
            let name = name.to_camel_case();
            let type_name = format!("{}Type", name);
            self.names.insert(&type_name).unwrap();
            
            let repr = iface
                .flags_repr(record)
                .expect("unsupported number of flags");

            self.src.go(&format!(
                "type {} {}\n\n",
                type_name,
                int_repr(repr),
            ));

            self.src.go("const (\n");
            for (i, field) in record.fields.iter().enumerate() {
                let field_name = format!("{}{}", name, field.name.to_camel_case());
                self.names.insert(&field_name).unwrap();

                if i == 0 {
                    self.src.go(&format!(
                        "\t{} {} = 1 << iota\n",
                        field_name,
                        type_name
                    ));
                } else {
                    self.src.go(&format!("\t{}\n", field_name));
                }
            }
            self.src.go(")\n\n"); // END: const (
        }

        // self.types
        //     .insert(id, mem::replace(&mut self.src.src, prev.src));
    }

    fn type_variant(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        _name: &str,
        _variant: &Variant,
        docs: &Docs,
    ) {
        self.docs(docs);

        unimplemented!("type_variant unimplemented")
    }

    fn type_resource(&mut self, iface: &Interface, ty: ResourceId) {
        drop((iface, ty));
    }

    fn type_alias(&mut self, _iface: &Interface, _id: TypeId, _name: &str, _ty: &Type, docs: &Docs) {
        self.docs(docs);

        unimplemented!("type_alias unimplemented")
    }

    fn type_list(&mut self, _iface: &Interface, _id: TypeId, _name: &str, _ty: &Type, docs: &Docs) {
        self.docs(docs);

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
        _iface: &Interface,
        _id: TypeId,
        _name: &str,
        _ty: &Type,
        _docs: &Docs,
    ) {
        unimplemented!("type_pull buffer unimplemented")
    }

    fn import(&mut self, iface: &Interface, func: &Function) {
        assert!(!func.is_async, "async not supported yet");

        println!("(import) iface:{}, func:{}", iface.name, func.name);
    }

    fn export(&mut self, _: &Interface, func: &Function) {
        assert!(!func.is_async, "async not supported yet");

        self.src.go(&format!("//export {}\n", func.name));
    }

    fn finish_one(&mut self, iface: &Interface, files: &mut Files) {
        let src = mem::take(&mut self.src);
        let name = iface.name.to_snake_case();

        files.push(&format!("{}/{}.go", name, name), src.src.as_bytes());
    }
}

impl Source {
    fn go(&mut self, s: &str) {
        self.src.push_str(s);
    }

    fn binding_header(&mut self, s: &str) {
        self.binding_header.push_str(s);
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

fn int_repr(ty: Int) -> &'static str {
    match ty {
        Int::U8 => "uint8",
        Int::U16 => "uint16",
        Int::U32 => "uint32",
        Int::U64 => "uint64",
    }
}