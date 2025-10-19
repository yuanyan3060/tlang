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

fn run() -> int {
    return 1;
}

fn main() {
    let h = Human("Bob", 18 + 1 * 2);
    print(h.name);
    print(h.get_age());
}