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

fn fibonacci(n: int) {
    if n <= 0 {
        return 0;
    } 

    if n == 1 {
        return 1;
    }
        
    let a = 0;
    let b = 1;
    let count = 2;

    while count <= n {
        let tmp = a + b;
        a = b;
        b = tmp;
        count =  count + 1;
    }
    return b;
}

fn main() {
    print(fibonacci(20));
    print("---------");
    let m = 0;
    while true {
        m = m + 1;
        if m % 2 == 1 {
            continue;
        }
        print("m =", m);
        if m > 20 {
            break;
        }
    }
    return;
}