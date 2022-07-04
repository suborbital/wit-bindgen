use heck::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::mem;
use std::process::{Command, Stdio};
use wit_bindgen_gen_core::wit_parser::abi::WasmType;
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
    types: HashMap<TypeId, wit_bindgen_gen_core::Source>,
}

// struct Func {
//     src: Source,
// }

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
pub struct Opts {
    /// Whether or not `gofmt` is executed to format generated code.
    #[cfg_attr(feature = "structopt", structopt(long))]
    pub gofmt: bool,
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

    fn print_ty(&mut self, iface: &Interface, ty: &Type) {
        match ty {
            Type::Id(id) => {
                let ty = &iface.types[*id];
                match &ty.name {
                    Some(name) => {
                        self.src.go(&name.to_camel_case());
                    }
                    None => match &ty.kind {
                        TypeDefKind::Type(t) => self.print_ty(iface, t),
                        TypeDefKind::Enum(_)
                        | TypeDefKind::Expected(_)
                        | TypeDefKind::Future(_)
                        | TypeDefKind::Option(_)
                        | TypeDefKind::Record(_)
                        | TypeDefKind::Stream(_)
                        | TypeDefKind::Tuple(_)
                        | TypeDefKind::Union(_)
                        | TypeDefKind::Variant(_) => {
                            unimplemented!();
                        }
                        TypeDefKind::Flags(_) => {
                            self.src.go("Type");
                        }
                        TypeDefKind::List(t) => {
                            self.src.go("[]");
                            self.print_ty(iface, t);
                        }
                    }
                }
            }
            Type::Handle(id) => {
                self.src.go(&iface.resources[*id].name.to_camel_case());
            }
            Type::Unit => self.src.go("struct{}"),
            Type::Bool => self.src.go("bool"),
            Type::Char => self.src.go("rune"),
            Type::U8 => self.src.go("byte"),
            Type::S8 => self.src.go("int8"),
            Type::U16 => self.src.go("uint16"),
            Type::S16 => self.src.go("int16"),
            Type::U32 => self.src.go("uint32"),
            Type::S32 => self.src.go("int32"),
            Type::U64 => self.src.go("uint64"),
            Type::S64 => self.src.go("int64"),
            Type::Float32 => self.src.go("float32"),
            Type::Float64 => self.src.go("float64"),
            Type::String => self.src.go("string"),
        }
    }

    fn print_anonymous_type(&mut self, iface: &Interface, ty: TypeId) {
        // let prev = mem::take(&mut self.src.go);

        let kind = &iface.types[ty].kind;
        match kind {
            TypeDefKind::Type(_)
            | TypeDefKind::Flags(_)
            | TypeDefKind::Record(_)
            | TypeDefKind::Enum(_)
            | TypeDefKind::Variant(_)
            | TypeDefKind::Union(_) => {
                unreachable!()
            }
            _ => {
                unimplemented!("anonymous types")
            }
        }
        // self.types
        //     .insert(ty, mem::replace(&mut self.src.go, prev));
    }

    fn is_empty_type(&self, iface: &Interface, ty: &Type) -> bool {
        let id = match ty {
            Type::Id(id) => *id,
            _ => return false,
        };
        match &iface.types[id].kind {
            TypeDefKind::Type(t) => self.is_empty_type(iface, t),
            TypeDefKind::Record(r) => r.fields.is_empty(),
            _ => false,
        }
    }
}

#[derive(Default)]
struct Source {
    go: wit_bindgen_gen_core::Source,
    // binding_header: wit_bindgen_gen_core::Source,
}

impl Source {
    fn go(&mut self, s: &str) {
        self.go.push_str(s);
    }

    // fn binding_header(&mut self, s: &str) {
    //     self.binding_header.push_str(s);
    // }
}

impl Generator for TinyGo {
    fn preprocess_one(&mut self, iface: &Interface, dir: Direction) {
        self.print_package(iface);
        drop(dir);
    }

    fn type_alias(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        _name: &str,
        _ty: &Type,
        docs: &Docs,
    ) {
        self.docs(docs);

        unimplemented!("type_alias")
    }

    fn type_record(
        &mut self,
        iface: &Interface,
        id: TypeId,
        name: &str,
        record: &Record,
        docs: &Docs,
    ) {
        let prev = mem::take(&mut self.src.go);
        self.docs(docs);
        let name = name.to_camel_case();

        self.names.insert(&name).unwrap();

        self.src.go(&format!("type {} struct {{\n", name,));
        for field in record.fields.iter() {
            self.src.go(&format!("\t{}", field.name.to_camel_case()));
            self.src.go(" ");
            self.print_ty(iface, &field.ty);
            self.src.go("\n");
        }
        self.src.go("}\n\n");

        self.types.insert(id, mem::replace(&mut self.src.go, prev));
    }

    fn type_enum(&mut self, _iface: &Interface, _id: TypeId, _name: &str, _enum_: &Enum, _docs: &Docs) {
        unimplemented!("type_enum")
    }

    fn type_expected(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        _name: &str,
        _expected: &Expected,
        _docs: &Docs,
    ) {
        unimplemented!("type_expected")
    }

    fn type_flags(
        &mut self,
        _iface: &Interface,
        id: TypeId,
        name: &str,
        flags: &Flags,
        docs: &Docs,
    ) {
        let prev = mem::take(&mut self.src.go);
        let type_name = format!("{}Type", name);
        self.docs(docs);

        self.names.insert(&type_name).unwrap();

        let repr = flags_repr(flags);

        self.src
            .go(&format!("type {} {}\n\n", type_name, int_repr(repr),));

        self.src.go("const (\n");
        for (i, flag) in flags.flags.iter().enumerate() {
            let field_name = format!("{}{}", name, flag.name.to_camel_case());
            self.names.insert(&field_name).unwrap();

            if i == 0 {
                self.src
                    .go(&format!("{} {} = 1 << iota\n", field_name, type_name));
            } else {
                self.src.go(&format!("{}\n", field_name));
            }
        }
        self.src.go(")\n\n");

        self.types.insert(id, mem::replace(&mut self.src.go, prev));
    }

    fn type_union(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        _name: &str,
        _union: &Union,
        _docs: &Docs,
    ) {
        unimplemented!("type_option")
    }

    fn type_option(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        _name: &str,
        _payload: &Type,
        _docs: &Docs,
    ) {
        unimplemented!("type_option")
    }

    fn type_tuple(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        _name: &str,
        _flags: &Tuple,
        _docs: &Docs,
    ) {
        unimplemented!("type_tuple")
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

        unimplemented!("type_variant")
    }

    fn type_resource(&mut self, iface: &Interface, ty: ResourceId) {
        drop((iface, ty));
    }

    fn type_list(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.docs(docs);

        if *ty == Type::Char {
            self.src.go("type ");
            self.src.go(&name.to_camel_case());
            self.src.go("string");
        } else {
            self.src.go("type ");
            self.src.go(&name.to_camel_case());
            self.src.go(" []");
            self.print_ty(iface, ty);
        }
        self.src.go("\n");
    }

    fn type_builtin(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        drop((iface, _id, name, ty, docs));
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
        let mut src = mem::take(&mut self.src.go);

        let name = iface.name.to_snake_case();

        for id in iface.topological_types() {
            if let Some(ty) = self.types.get(&id) {
                src.push_str(ty);
            }
        }

        if self.opts.gofmt {
            let mut child = Command::new("gofmt")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to spawn `gofmt`");
            child
                .stdin
                .take()
                .unwrap()
                .write_all(src.as_bytes())
                .unwrap();
            src.as_mut_string().truncate(0);
            child
                .stdout
                .take()
                .unwrap()
                .read_to_string(src.as_mut_string())
                .unwrap();
            let status = child.wait().unwrap();
            assert!(status.success());
        }

        files.push(&format!("{}/{}.go", name, name), src.as_bytes());
    }
}

// fn wasm_type(ty: WasmType) -> &'static str {
//     match ty {
//         WasmType::I32 => "int",
//         WasmType::I64 => "int64",
//         WasmType::F32 => "float32",
//         WasmType::F64 => "float64",
//     }
// }

fn int_repr(ty: Int) -> &'static str {
    match ty {
        Int::U8 => "uint8",
        Int::U16 => "uint16",
        Int::U32 => "uint32",
        Int::U64 => "uint64",
    }
}

fn flags_repr(f: &Flags) -> Int {
    match f.repr() {
        FlagsRepr::U8 => Int::U8,
        FlagsRepr::U16 => Int::U16,
        FlagsRepr::U32(1) => Int::U32,
        FlagsRepr::U32(2) => Int::U64,
        repr => panic!("unimplemented flags {:?}", repr),
    }
}
