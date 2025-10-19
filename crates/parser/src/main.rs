fn main() {
    let a = "你好\n世界";
    let x = [1, 2, 3, 4, 5, 6];
    let mut x = x.iter();
    x.next();
    println!("{:?}", x.as_slice());
    println!("{}", a)
}
