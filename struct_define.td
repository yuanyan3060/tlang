fn main() {
    let arr = Vec<int>::new();
    arr.push(0);
    arr.push(1);
    arr.push(2);
    print(arr);

    let arrs = Vec<Vec<int>>::new();
    arrs.push(arr);
    arrs.push(arr);
    arrs.push(arrs);
    print(arrs);
    return;
}
