fn fib(n: int) -> int {
    let a = 0;
    let b = 1;

    while(n > 0) {
        let temp = a;
        a = b;
        b = temp + b;
        n = n - 1;
    }

    return a;
}

fn main() {
    let n = 10000000;
    while(n != 0) {
        fib(10);
        n = n - 1;
    }
}