use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum Object {
    Fn(u32),
    Struct(u32),
}

pub struct Scope {
    pub name: String,
    pub idx: usize,
    pub parent: usize,
    pub children: HashMap<String, usize>,
    pub objs: HashMap<String, Object>,
}

pub struct Env {
    pub scopes: Vec<Scope>,
    pub curr: usize,
    pub namespace: Vec<String>,
    pub externs: HashMap<String, Env>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope {
                name: "".to_string(),
                idx: 0,
                parent: 0,
                children: HashMap::new(),
                objs: HashMap::new(),
            }],
            curr: 0,
            namespace: Vec::new(),
            externs: HashMap::new(),
        }
    }

    pub fn add_fn(&mut self, name: &str, id: u32) {
        let curr = &mut self.scopes[self.curr];
        curr.objs.insert(name.to_string(), Object::Fn(id));
    }

    pub fn add_struct(&mut self, name: &str, id: u32) {
        let curr = &mut self.scopes[self.curr];
        curr.objs.insert(name.to_string(), Object::Struct(id));
    }

    pub fn lookup(&self, path: &[ast::PathSegment]) -> Option<(usize, &Object)> {
        let mut curr = &self.scopes[self.curr];
        for (i, seg) in path.iter().enumerate() {
            let last = i + 1 == path.len();
            match seg.ident.as_str() {
                "super" => {
                    if curr.idx == 0 {
                        return None;
                    }
                    curr = &self.scopes[curr.parent];
                    continue;
                }
                "crate" => {
                    curr = &self.scopes[0];
                    continue;
                }
                _ if last => {
                    let obj = curr.objs.get(&seg.ident)?;
                    return Some((curr.idx, obj));
                }
                _ => {
                    let idx = *curr.children.get(&seg.ident)?;
                    curr = &self.scopes[idx];
                }
            }
        }

        None
    }

    pub fn lookup_scope(&self, path: &[ast::PathSegment]) -> Option<&Scope> {
        let mut curr = &self.scopes[self.curr];

        if path.is_empty() {
            return Some(curr);
        }

        for (i, seg) in path.iter().enumerate() {
            let last = i + 1 == path.len();
            match seg.ident.as_str() {
                "super" => {
                    if curr.idx == 0 {
                        return None;
                    }
                    curr = &self.scopes[curr.parent];
                    continue;
                }
                "crate" => {
                    curr = &self.scopes[0];
                    continue;
                }
                _ => {
                    let idx = *curr.children.get(&seg.ident)?;
                    curr = &self.scopes[idx];

                    if last {
                        return Some(curr);
                    }
                }
            }
        }

        None
    }

    pub fn lookup_scope_mut(&mut self, path: &[ast::PathSegment]) -> Option<&mut Scope> {
        let idx = self.lookup_scope(path)?.idx;
        self.scopes.get_mut(idx)
    }

    pub fn enter(&mut self, name: &str) {
        let idx = self.scopes.len();
        let curr = &mut self.scopes[self.curr];
        curr.children.insert(name.to_string(), idx);

        let scope = Scope {
            name: name.to_string(),
            idx,
            parent: self.curr,
            children: HashMap::new(),
            objs: HashMap::new(),
        };

        self.scopes.push(scope);
        self.curr = idx;
        self.namespace.push(name.to_string());
    }

    pub fn exit(&mut self) {
        let curr = &self.scopes[self.curr];
        self.curr = curr.parent;
        self.namespace.pop();
    }

    pub fn goto_root(&mut self) {
        self.curr = 0;
    }

    pub fn full_name(&self, name: &str) -> String {
        if self.namespace.is_empty() {
            return name.to_string();
        }

        let mut full = self.namespace.join("::");
        full.push_str("::");
        full.push_str(name);
        full
    }
}

#[cfg(test)]
mod test {
    use crate::scope::Env;

    #[test]
    fn env_test() {
        let mut env = Env::new();

        env.enter("hello");
        println!("{:?}", env.namespace);

        env.enter("world");
        println!("{:?}", env.namespace);

        env.exit();
        println!("{:?}", env.namespace);
    }
}
