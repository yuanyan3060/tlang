struct Human {
    name: str
    age: int

    fn new(self, name: str, age: int) {
        self.name = name;
        self.age = age;
    }

    fn get_age(self) -> int {
        return self.age;
    }
}

fn run(a: int, b: int) -> int {
    return a + b;
}

fn main() {
    let h = Human("Rust", 10);
    print(h.name);
    print(run(h.get_age(), h.age));
}