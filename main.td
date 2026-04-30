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