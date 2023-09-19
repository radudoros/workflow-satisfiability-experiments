use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone)]
pub struct Graph {
    // labels:
    pub nodes_id: Vec<i32>,
    // graph edges: (note that they're a DAG always)
    pub adjacency_list: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new(sz: usize) -> Graph {
        Graph {
            nodes_id: vec![-1; sz],
            adjacency_list: vec![Vec::new(); sz],
        }
    }

    pub fn print(&self) {
        print!("Ids: ");
        for id in self.nodes_id.iter() {
            print!("{} ", id);
        }
        println!();
        println!("Adjacency List: ");
        for (i, adj) in self.adjacency_list.iter().enumerate() {
            print!("For node {} we have children: ", i);
            for ch in adj.iter() {
                print!("{} ", ch);
            }
            println!();
        }
    }

    pub fn from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Open the file for reading
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        // Read the number of nodes from the first line
        let mut lines = reader.lines();
        let n: usize = lines.next().unwrap()?.trim().parse()?;

        // Initialize an empty graph with n nodes
        let mut g = Graph::new(n);

        // Read edges from the remaining lines and add them to the graph
        for line in lines {
            let nums: Vec<usize> = line?
                .trim()
                .split_whitespace()
                .map(|num| num.parse().unwrap())
                .collect();

            g.add_edge(nums[0], nums[1]);
        }

        Ok(g)
    }

    pub fn add_edge(&mut self, parent: usize, child: usize) {
        self.adjacency_list[usize::from(parent)].push(child);
    }
}

#[allow(dead_code)]
fn topo_sort_helper(g: &Graph, v: usize, topo_sorted: &mut Vec<usize>, visited: &mut Vec<bool>) {
    if visited[v] {
        return;
    }

    visited[v] = true;
    for ch in g.adjacency_list[v].iter() {
        topo_sort_helper(g, *ch, topo_sorted, visited);
    }

    topo_sorted.push(v)
}

#[allow(dead_code)]
pub fn topo_sort(g: &Graph) -> Vec<usize> {
    let mut visited: Vec<bool> = vec![false; g.nodes_id.len()];
    let mut topo_sorted: Vec<usize> = Vec::new();

    for i in 0..g.nodes_id.len() {
        topo_sort_helper(g, i, &mut topo_sorted, &mut visited);
    }

    topo_sorted
}
