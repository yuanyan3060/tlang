pub mod generator;

#[derive(Debug)]
pub enum ByteCode {
    LoadNil,
    LoadBool { val: bool },
    LoadInt { val: i64 },
    LoadFloat { val: f64 },
    LoadString { idx: u32 },
    LoadFunction { idx: u32 },
    NewStruct { idx: u32, cnt: u32 },
    // 把栈顶的元素弹出 存入局部变量表下标为 idx 位置
    Store { idx: u32 },
    // 把 idx 下标的局部变量压入栈顶
    Load { idx: u32 },
    GetField { offset: u32 },
    SetField { offset: u32 },
    Add,
    Multiply,
    GetIndex,
    Call { param_cnt: u16 },
    // 把栈顶两个元素交换
    Swap,
    Return,
    Pop,
}
