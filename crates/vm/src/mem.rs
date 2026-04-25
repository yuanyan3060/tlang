use std::array::from_fn;
use std::collections::VecDeque;

use crate::value::{GcData, GcHandle, GcMark, GcValue, Value};
use slotmap::SlotMap;

pub struct Mem {
    pub stack: Vec<Value>,
    pub local: Vec<Value>,
    pub temp: Vec<Value>,
    pub global: Vec<Value>,
    pub consts: Vec<Value>,
    pub heap: SlotMap<GcHandle, GcValue>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            local: Vec::new(),
            temp: Vec::new(),
            global: Vec::new(),
            consts: Vec::new(),
            heap: SlotMap::default(),
        }
    }

    #[inline(always)]
    pub fn get(&self, handle: GcHandle) -> &GcData {
        self.heap.get(handle).map(|x| &x.data).unwrap()
    }

    #[inline(always)]
    pub fn get_mut(&mut self, handle: GcHandle) -> &mut GcData {
        self.heap.get_mut(handle).map(|x| &mut x.data).unwrap()
    }

    #[inline(always)]
    pub fn alloc(&mut self, data: GcData) -> GcHandle {
        self.heap.insert(GcValue {
            data,
            mark: GcMark::White,
        })
    }

    #[inline(always)]
    pub fn push_stack(&mut self, value: Value) {
        self.stack.push(value);
    }
}

pub struct Tracer {
    queue: VecDeque<GcHandle>,
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn start_gc(&mut self, mem: &mut Mem) {
        self.mark(mem);
        self.clean(mem);
    }

    fn mark(&mut self, mem: &mut Mem) {
        self.queue.clear();

        let root = mem
            .stack
            .iter()
            .chain(&mem.global)
            .chain(&mem.local)
            .chain(&mem.temp)
            .chain(&mem.consts);

        for value in root {
            let Some(handle) = value.as_handle() else {
                continue;
            };

            mem.heap.get_mut(handle).unwrap().mark = GcMark::Gray;
            self.queue.push_back(handle);
        }

        while let Some(handle) = self.queue.pop_front() {
            let value = mem.heap.get_mut(handle).unwrap();
            if value.mark != GcMark::Gray {
                continue;
            }
            value.mark = GcMark::Black;

            let mut size = 0;
            match &value.data {
                GcData::String(_) => {}
                GcData::Vec(values) | GcData::Struct(values) => {
                    for value in values {
                        let Some(handle) = value.as_handle() else {
                            continue;
                        };
                        self.queue.push_back(handle);
                        size += 1;
                    }
                }
            }

            let start = self.queue.len() - size;
            let end = self.queue.len();

            for i in start..end {
                let handle = self.queue.get(i).unwrap();
                let field = mem.heap.get_mut(*handle).unwrap();
                if field.mark == GcMark::White {
                    field.mark = GcMark::Gray;
                }
            }
        }
    }

    fn clean(&self, mem: &mut Mem) {
        mem.heap.retain(|_, value| {
            let clean = value.mark != GcMark::White;
            value.mark = GcMark::White;
            clean
        });
    }
}
