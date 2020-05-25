extern crate serde_json;
extern crate wyvern_core;

use wyvern_core::builder::ProgramBuilder;
use wyvern_core::types::{Array, Constant, Variable};

fn main() {
    let builder = ProgramBuilder::new();
    let _100 = Constant::new(100, &builder);
    let _20 = Constant::new(20, &builder);
    let _a = Variable::new(&builder).mark_as_output("ciaone");
    let _b: Variable<u32> = Variable::new(&builder);
    _a.store(_20);
    let _b = _a.load();
    _a.store(builder.worker_id());
    let _c = _20 - _b;
    let v: Array<u32> = Array::new(_100, 120, true, &builder).mark_as_output("lollone");
    let _xv: Array<u32> = Array::new(_20, 240, false, &builder);
    builder.if_then(
        |_| _20.lt(_100),
        |ctx| ctx.while_loop(|_| _20.lt(_100), |_| v.at(_20).store(_100)),
    );
    let _a = v.at(_20).load();
    let program = builder.finalize().unwrap();
    println!("{}", serde_json::to_string_pretty(&program).unwrap());
}
