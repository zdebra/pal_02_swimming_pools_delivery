use std::collections::LinkedList;
use std::{io, vec};

// const UNVISITED: isize = -1;

fn main() {
    let mut lines = io::stdin().lines();
    let header = lines.next().unwrap().unwrap();

    let mut header_splits = header.split_whitespace();
    let num_crossings = header_splits.next().unwrap().parse::<usize>().unwrap(); // number of nodes
    let num_streets = header_splits.next().unwrap().parse::<usize>().unwrap(); // number of edges

    println!("num crossings: {}", num_crossings);
    println!("num streets: {}", num_streets);

    let mut adj_list = vec![LinkedList::<usize>::new(); num_crossings];

    lines.for_each(|line| {
        let lu = line.unwrap();
        let mut line_splits = lu.split_whitespace();
        let from = line_splits.next().unwrap().parse::<usize>().unwrap();
        let to = line_splits.next().unwrap().parse::<usize>().unwrap();
        adj_list[from].push_back(to);
    });

    for (i, from) in adj_list.iter().enumerate() {
        from.iter().for_each(|to| println!("from {} to {}", i, to));
    }

    let mut graph = Network::new(&adj_list, num_crossings, num_streets);
    let out = graph.find_sccs();
    for i in out {
        print!("{} ", i);
    }
    println!("");
    println!("{}", graph.scc_count);
}

#[derive(Clone, PartialEq, Copy)]
enum NodeID {
    Unvisited,
    Visited(isize),
}

impl NodeID {
    fn must_get(&self) -> isize {
        match *self {
            Self::Unvisited => panic!("get value for unvisited node"),
            Self::Visited(v) => v,
        }
    }
}

struct Network<'a> {
    num_crossings: usize,
    num_streets: usize,
    adj_list: &'a Vec<LinkedList<usize>>,
    ids: Vec<NodeID>,
    low: Vec<isize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    id: isize,
    scc_count: isize,
}

impl<'a> Network<'a> {
    fn new(adj_list: &'a Vec<LinkedList<usize>>, num_crossings: usize, num_streets: usize) -> Self {
        Self {
            num_crossings,
            num_streets,
            adj_list,
            ids: vec![NodeID::Unvisited; num_crossings],
            low: vec![0; num_crossings],
            on_stack: vec![false; num_crossings],
            stack: Vec::new(),
            id: 0,
            scc_count: 0,
        }
    }

    fn find_sccs(&mut self) -> &Vec<isize> {
        for node in 0..self.num_crossings {
            if self.ids[node] == NodeID::Unvisited {
                self.dfs(node);
            }
        }
        &self.low
    }

    fn dfs(&mut self, at: usize) {
        self.stack.push(at);
        self.on_stack[at] = true;

        let id = self.get_id();
        self.ids[at] = id;
        self.low[at] = id.must_get();

        let neighbours = &self.adj_list[at];
        for &to in neighbours.iter() {
            if self.ids[to] == NodeID::Unvisited {
                self.dfs(to);
            }
            if self.on_stack(to) {
                self.low[at] = std::cmp::min(self.low[at], self.low[to]);
            }
        }

        if self.ids[at].must_get() != self.low[at] {
            return;
        }
        loop {
            let node = match self.stack.pop() {
                Some(v) => v,
                None => break,
            };
            self.on_stack[node] = false;
            self.low[node] = self.ids[at].must_get();
            if node == at {
                break;
            }
        }
        self.scc_count += 1;
    }

    fn get_id(&mut self) -> NodeID {
        let cur_id = self.id;
        self.id += 1;
        return NodeID::Visited(cur_id);
    }

    fn on_stack(&self, to: usize) -> bool {
        self.on_stack[to]
    }
}
