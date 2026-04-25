use std::array::from_fn;
use std::collections::VecDeque;

use slotmap::SlotMap;
use crate::value::{GcData, GcHandle, GcMark, GcValue, Value};

pub struct Mem {
    pub stack: Vec<Value>,
    pub global: Vec<Value>,
    pub heap: SlotMap<GcHandle, GcValue>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            global: Vec::new(),
            heap: SlotMap::default(),
        }
    }

    pub fn alloc(&mut self, data: GcData) -> GcHandle {
        self.heap.insert(GcValue {
            data,
            mark: GcMark::White,
        })
    }

    pub fn push_stack(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop_stack(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn batch_pop_stack<const N: usize>(&mut self) -> Option<[Value; N]> {
        if self.stack.len() < N {
            return None;
        }

        let len = self.stack.len();
        let start = len.saturating_sub(N);

        let mut iter = self.stack.drain(start..);
        
        Some(std::array::from_fn(|_| {
            iter.next().unwrap()
        }))
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

        for value in mem.stack.iter().chain(&mem.global) {
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
