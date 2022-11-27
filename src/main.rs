use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;
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

    println!("variability");
    for crossing in 0..graph.num_crossings {
        let group: usize = graph.low[crossing].try_into().unwrap();
        let v: usize = graph
            .strong_crossing_group_items_counter
            .get(&group)
            .unwrap_or(&1)
            - 1;
        println!("variability of {} is {}", crossing, v);
    }
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
    is_strong_crossing: Vec<bool>,
    strong_crossing_group_items_counter: HashMap<usize, usize>,
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
            is_strong_crossing: vec![false; num_crossings],
            strong_crossing_group_items_counter: HashMap::new(),
        }
    }

    fn find_sccs(&mut self) -> &Vec<isize> {
        for node in 0..self.num_crossings {
            if self.ids[node] == NodeID::Unvisited {
                self.dfs(node);
            }
        }
        // scc is done

        // find all strong crossings
        for crossing in 0..self.num_crossings {
            let crossing_group = self.low[crossing];
            let neighbours = &self.adj_list[crossing];
            let is_strong_crossing = neighbours
                .iter()
                .all(|&neighbour| self.low[neighbour] == crossing_group);
            self.is_strong_crossing[crossing] = is_strong_crossing;

            // count strong crossings in the group
            if is_strong_crossing {
                let c: usize = crossing_group.try_into().unwrap();
                println!("c = {}", c);
                let count = self
                    .strong_crossing_group_items_counter
                    .entry(c)
                    .or_insert(0);
                *count += 1;
            }
        }

        let mut target_costs: HashMap<usize, HashMap<usize, usize>> = HashMap::new();
        let mut iterated: HashSet<isize> = HashSet::new();
        for crossing in 0..self.num_crossings {
            if !self.is_strong_crossing[crossing] {
                continue;
            }

            let crossing_group = self.low[crossing];
            if iterated.contains(&crossing_group) {
                continue;
            }

            // todo
            let mut q = vec![(crossing, self.neighbours_cost(crossing))]; // (crossing, updates)
            loop {
                let c = match q.pop() {
                    None => break,
                    Some(c) => c,
                };

                let crossing_targets = target_costs.entry(c.0).or_insert(HashMap::new());
                let mut mutated = false;
                // iterate updates
                for (crossing, cost) in c.1 {
                    let old_cost = crossing_targets.entry(crossing).or_insert(cost);
                    if cost < *old_cost {
                        *old_cost = cost;
                        mutated = true;
                    }
                }

                if !mutated {
                    continue;
                }

                // publish updates for all strong neighbours
                for neighbour in self.adj_list[crossing]
                    .into_iter()
                    .filter(|&x| self.is_strong_crossing[x])
                {
                    let updates = self.targets_to_updates(crossing_targets);
                    q.push((neighbour, updates));
                }
            }

            iterated.insert(crossing_group);
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

    fn neighbours_cost(&self, crossing: usize) -> HashMap<usize, usize> {
        let mut hm = HashMap::new();
        let neighbours = &self.adj_list[crossing];
        for neighbour in neighbours
            .into_iter()
            .filter(|&&x| self.is_strong_crossing[x])
        {
            hm.insert(*neighbour, 1);
        }
        hm
    }

    fn targets_to_updates(
        &self,
        crossing_targets: &HashMap<usize, usize>,
    ) -> HashMap<usize, usize> {
        let mut hm = HashMap::new();
        for (k, v) in crossing_targets {
            hm.insert(k, v + 1);
        }
        hm
    }
}
