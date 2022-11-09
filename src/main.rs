use std::io;
fn main() {
    let mut lines = io::stdin().lines();
    let header = lines.next().unwrap().unwrap();

    let mut header_splits = header.split_whitespace();
    let num_crossings = header_splits.next().unwrap().parse::<u32>().unwrap();
    let num_streets = header_splits.next().unwrap().parse::<u32>().unwrap();

    println!("num crossings: {}", num_crossings);
    println!("num streets: {}", num_streets);

    lines.for_each(|line| {
        let lu = line.unwrap();
        let mut line_splits = lu.split_whitespace();
        let from = line_splits.next().unwrap().parse::<u32>().unwrap();
        let to = line_splits.next().unwrap().parse::<u32>().unwrap();
        println!("from: {}, to: {}", from, to);
    });
}
