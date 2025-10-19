use vm::Vm;

fn main() {
    let mut vm = Vm::new();
    vm.load_file("struct_define.td").unwrap();
}
