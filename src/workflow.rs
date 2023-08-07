#[derive(Clone)]
pub struct graph {
    // labels:
    pub nodes_id: Vec<i32>,
    // graph edges: (note that they're a DAG always)
    pub adjacency_list: Vec<Vec<usize>>
}

impl graph {
    pub fn new(sz: usize) -> graph {
        graph {
            nodes_id: vec![-1; sz],
            adjacency_list:  vec![Vec::new(); sz]
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

    pub fn add_edge(&mut self, parent: usize, child: usize) {
        self.adjacency_list[usize::from(parent)].push(child);
    }
}

fn topo_sort_helper(g: &graph, v: usize, topo_sorted: &mut Vec<usize>, visited: &mut Vec<bool>) {
    if visited[v] {return;}

    visited[v] = true;
    for ch in g.adjacency_list[v].iter() {
        topo_sort_helper(g, *ch, topo_sorted, visited);
    }

    topo_sorted.push(v)
}

pub fn topo_sort(g: &graph) -> Vec<usize> {
    let mut visited: Vec<bool> = vec![false; g.nodes_id.len()];
    let mut topo_sorted: Vec<usize> = Vec::new();

    for i in 0..g.nodes_id.len() {
        topo_sort_helper(g, i, &mut topo_sorted, &mut visited);
    }

    topo_sorted
}
