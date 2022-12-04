use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::isize::MAX;
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
    let mut target_list = vec![LinkedList::<usize>::new(); num_crossings]; // = the opossite of adj_list

    lines.for_each(|line| {
        let lu = line.unwrap();
        let mut line_splits = lu.split_whitespace();
        let from = line_splits.next().unwrap().parse::<usize>().unwrap();
        let to = line_splits.next().unwrap().parse::<usize>().unwrap();
        adj_list[from].push_back(to);
        target_list[to].push_back(from);
    });

    let mut graph = Network::new(&adj_list, &target_list, num_crossings);
    graph.find_sccs();

    println!("variability");
    let mut max_variability = 0;
    let mut max_variability_list = Vec::<usize>::new();
    for crossing in 0..graph.num_crossings {
        let group: usize = graph.low[crossing].try_into().unwrap();
        let v: usize = graph
            .strong_crossing_group_items_counter
            .get(&group)
            .unwrap_or(&1)
            - 1;
        if v == max_variability {
            max_variability_list.push(crossing);
        } else if v > max_variability {
            max_variability = v;
            max_variability_list.clear();
            max_variability_list.push(crossing);
        }
    }

    println!(
        "max variability is {max_variability} with these crossings {:?}",
        max_variability_list
    );

    println!("");
    println!("costs:");
    let mut min_cost = usize::MAX;
    let mut min_cost_list = Vec::<usize>::new();
    for crossing in max_variability_list {
        if !graph.is_strong_crossing[crossing] {
            continue;
        }
        let mut sum = 0;
        for (target, cost) in graph.crossing_target_cost.get(&crossing).unwrap() {
            if target == &crossing {
                continue;
            }
            if !graph.is_strong_crossing[*target] {
                continue;
            }
            let cost_for_return = graph
                .crossing_target_cost
                .get(target)
                .unwrap()
                .get(&crossing)
                .unwrap();

            sum += 2 * cost;
            sum += cost_for_return;
        }

        if sum == min_cost {
            min_cost_list.push(crossing);
        } else if sum < min_cost {
            min_cost = sum;
            min_cost_list.clear();
            min_cost_list.push(crossing);
        }
    }

    println!("min cost is {min_cost} for these {:?}", min_cost_list);
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
    adj_list: &'a Vec<LinkedList<usize>>,
    target_list: &'a Vec<LinkedList<usize>>,
    ids: Vec<NodeID>,
    low: Vec<isize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    id: isize,
    scc_count: isize,
    is_strong_crossing: Vec<bool>,
    strong_crossing_group_items_counter: HashMap<usize, usize>,
    crossing_target_cost: HashMap<usize, HashMap<usize, usize>>,
}

impl<'a> Network<'a> {
    fn new(
        adj_list: &'a Vec<LinkedList<usize>>,
        target_list: &'a Vec<LinkedList<usize>>,
        num_crossings: usize,
    ) -> Self {
        Self {
            num_crossings,
            adj_list,
            target_list,
            ids: vec![NodeID::Unvisited; num_crossings],
            low: vec![0; num_crossings],
            on_stack: vec![false; num_crossings],
            stack: Vec::new(),
            id: 0,
            scc_count: 0,
            is_strong_crossing: vec![false; num_crossings],
            strong_crossing_group_items_counter: HashMap::new(),
            crossing_target_cost: HashMap::new(),
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
                let count = self
                    .strong_crossing_group_items_counter
                    .entry(c)
                    .or_insert(0);
                *count += 1;
            }
        }

        println!("starting to compute distance cost");

        let mut iterated: HashSet<isize> = HashSet::new();
        for crossing in 0..self.num_crossings {
            if !self.is_strong_crossing[crossing] {
                continue;
            }

            // iterate each scs only once
            let crossing_group = self.low[crossing];
            if iterated.contains(&crossing_group) {
                continue;
            }

            // send all targets of this crossing to all nodes this crossing is target of
            struct Update {
                crossing: usize,
                updates: HashMap<usize, usize>,
            }

            let mut q = vec![Update {
                crossing,
                updates: HashMap::<usize, usize>::new(),
            }];
            loop {
                // println!("cur queue size is {}", q.len());
                let u = match q.pop() {
                    None => break,
                    Some(u) => u,
                };
                // println!("got {} updates", u.updates.len());

                // 1. get my targets
                let my_targets = self
                    .crossing_target_cost
                    .entry(u.crossing)
                    .or_insert(HashMap::new());

                let mut mutated = false;
                // 2. update targets with my strong neighbours from the same scc
                for n in &self.adj_list[u.crossing] {
                    if !self.is_strong_crossing[*n] {
                        continue;
                    }
                    if self.low[u.crossing] != self.low[*n] {
                        continue;
                    }
                    if let Some(&cost) = my_targets.get(n) {
                        if cost > 1 {
                            mutated = true;
                            my_targets.insert(*n, 1);
                        }
                        // else we already have this value
                    } else {
                        mutated = true;
                        my_targets.insert(*n, 1);
                    }
                }

                // 3. apply updates
                for (target_crossing, cost) in u.updates.into_iter() {
                    if let Some(&old_cost) = my_targets.get(&target_crossing) {
                        if cost < old_cost {
                            mutated = true;
                            my_targets.insert(target_crossing, cost);
                        }
                    } else {
                        mutated = true;
                        my_targets.insert(target_crossing, cost);
                    }
                }

                // 4. send updates to all targets of this crossing
                if !mutated {
                    continue;
                }

                let mut next_updates = HashMap::<usize, usize>::new();
                for (target_crossing, cur_cost) in my_targets {
                    next_updates.insert(*target_crossing, *cur_cost + 1);
                }

                for &target_of in &self.target_list[u.crossing] {
                    if !self.is_strong_crossing[target_of] {
                        continue;
                    }
                    if self.low[target_of] != self.low[u.crossing] {
                        continue;
                    }
                    q.push(Update {
                        crossing: target_of,
                        updates: next_updates.clone(),
                    })
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
}
