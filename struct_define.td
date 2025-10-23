struct Human {
    name: str
    age: int

    fn new(name: str, age: int) -> Human {
        return Human {
            name: name,
            age: age
        }
    }

    fn get_age(self) -> int {
        return self.age;
    }

    fn add_age(self, a: int) -> int {
        self.age = self.age + a;
        return self.age;
    }

    fn to_str(self) -> str {
        return "Human(name: \"{}\", age: {})".format(self.name, self.age);
    }
}

fn main() {
    let h = Human::new("name", 10);
    print(h.to_str());
    return;
}