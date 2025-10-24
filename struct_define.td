struct Human {
    name: str,
    age: int,
}

impl Human {
    fn new(name: str, age: int) -> Human {
        return Human {
            name: name,
            age: age,
        };
    }

    fn add_age(self, val: int) -> int {
        self.age = self.age + val;
        return self.age;
    }

    fn to_str(self) -> str {
        return "Human(name: \"{}\", age: {})".format(self.name, self.age);
    }
}

fn main() {
    let h = Human::new("1", 1);
    print(h.add_age(1));
    print(h.to_str());
    return;
}
