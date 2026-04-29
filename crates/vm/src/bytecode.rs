use std::fmt::Debug;

use crate::ir::Variable;

#[derive(Debug)]
pub enum ByteCode {
    // --- 一元运算 (Unary Operations) ---
    /// dst = +src
    Pos {
        dst: Loc,
        src: Loc,
    },
    /// dst = -src
    Neg {
        dst: Loc,
        src: Loc,
    },
    /// dst = !src (逻辑取反)
    Not {
        dst: Loc,
        src: Loc,
    },
    /// dst = ^src (按位取反)
    BitNot {
        dst: Loc,
        src: Loc,
    },

    // --- 二元运算 (算术运算) ---
    /// dst = left + right
    Add {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left - right
    Sub {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left * right
    Mul {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left / right
    Div {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left % right
    Mod {
        dst: Loc,
        left: Loc,
        right: Loc,
    },

    // --- 二元运算 (位运算) ---
    /// dst = left & right (按位与)
    BitAnd {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left | right (按位或)
    BitOr {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left ^ right (按位异或)
    BitXor {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left << right (左移)
    Shl {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left >> right (右移)
    Shr {
        dst: Loc,
        left: Loc,
        right: Loc,
    },

    // --- 二元运算 (比较运算) ---
    /// dst = (left == right)
    Eq {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = (left != right)
    Ne {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = (left < right)
    Lt {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = (left <= right)
    Le {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = (left > right)
    Gt {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = (left >= right)
    Ge {
        dst: Loc,
        left: Loc,
        right: Loc,
    },

    // --- 二元运算 (逻辑运算) ---
    /// dst = left && right
    And {
        dst: Loc,
        left: Loc,
        right: Loc,
    },
    /// dst = left || right
    Or {
        dst: Loc,
        left: Loc,
        right: Loc,
    },

    GetParam {
        dst: Loc,
    },

    Param {
        src: Loc,
    },

    NewObject {
        dst: Loc,
        size: u32,
    },

    Call {
        dst: Loc,
        func: Loc,
        param_cnt: u16,
    },

    Load {
        from: Loc,
        to: Loc,
    },
    Br {
        cond: Loc,
        then_offset: u32,
        else_offset: u32,
    },
    Jump {
        offset: u32,
    },
    SetIndex {
        dst: Loc,
        idx: Loc,
        src: Loc,
    },
    Index {
        dst: Loc,
        idx: Loc,
        src: Loc,
    },
    SetMember {
        dst: Loc,
        offset: u16,
        src: Loc,
    },
    Member {
        dst: Loc,
        offset: u16,
        src: Loc,
    },
    Return {
        src: Option<Loc>,
    },
}

#[derive(Clone, Copy)]
pub struct Loc(pub(crate) u16);

impl Debug for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = self.0 >> 14;
        let idx = (self.0 & 0x3fff) as usize;
        match tag {
            0 => write!(f, "local({})", idx),
            1 => write!(f, "temp({})", idx),
            2 => write!(f, "global({})", idx),
            3 => write!(f, "consts({})", idx),
            _ => unreachable!(),
        }
    }
}

impl Loc {
    pub fn from_const(idx: usize) -> Self {
        debug_assert!(idx < 0x4000, "Constant index out of bounds (14-bit limit)");
        // 0xC000 是二进制 1100 0000 0000 0000
        Self(0xC000 | (idx as u16))
    }
}

impl From<Variable> for Loc {
    fn from(value: Variable) -> Self {
        match value {
            Variable::Local(idx) => {
                debug_assert!(idx < 0x4000, "Local index out of bounds (14-bit limit)");
                Loc(idx as u16)
            }
            Variable::Temp(idx) => {
                debug_assert!(idx < 0x4000, "Temp index out of bounds (14-bit limit)");
                Loc(0x4000 | (idx as u16))
            }
            Variable::Global(idx) => {
                debug_assert!(idx < 0x4000, "Global index out of bounds (14-bit limit)");
                Loc(0x8000 | (idx as u16))
            }
        }
    }
}
