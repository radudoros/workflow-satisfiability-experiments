
struct graph {
    // labels:
    nodes_id: Vec<i32>,
    // graph edges: (note that they're a DAG always)
    adjacency_list: Vec<Vec<usize>>
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

struct predicates {
    preds: Vec<Box<dyn Fn(&graph) -> bool>>,
    // predicate location id (to identify sites' predicates)
    pred_loc: Vec<i32>
}

impl predicates {
    pub fn eval(&self, g: &graph) -> bool {
        for p in self.preds.iter() {
            if !p(g) {
                return false;
            }
        }

        true
    }

    pub fn filtered_eval(&self, g: &graph, filtered_in: i32) -> bool {
        for (i, p) in self.preds.iter().enumerate() {
            if self.pred_loc[i] != filtered_in {continue;}
            
            if !p(g) {
                return false;
            }
        }

        true
    }
}

fn generate_mock_ui_predicates() -> predicates {
    let mut v: Vec<Box<dyn Fn(&graph) -> bool>> = Vec::new();
    // v.push(Box::new(|_| true));
    v.push(Box::new(|g:&graph| {
        // policy: don't allow a step and its child step to be same:
        for (v, adj) in g.adjacency_list.iter().enumerate() {
            if g.nodes_id[v] == -1 {continue;}
            for neigh in adj.iter() {
                if g.nodes_id[v] == g.nodes_id[*neigh] {return false;}
            }
        }

        return true;
    }));

    let mut ids:Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i)
    }

    predicates { preds: v, pred_loc: ids }
}

fn generate_mock_ud_predicates() -> predicates {
    let mut v: Vec<Box<dyn Fn(&graph) -> bool>> = Vec::new();

    v.push(Box::new(|_| true));
    v.push(Box::new(|g:&graph| {
        g.nodes_id.len() < 2 || g.nodes_id[2] == -1 || g.nodes_id[2] == 1
    }));
    v.push(Box::new(|g:&graph| {
        g.nodes_id.len() < 1 || g.nodes_id[1] == -1 || g.nodes_id[1] == 2
    }));

    let mut ids:Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i)
    }

    predicates { preds: v, pred_loc: ids }
}


// Simple backtracking algorithm:
fn plan(g: &mut graph, i: usize, node_options: &Vec<i32>, preds : &predicates, sol: &mut Vec<i32>) {
    if !sol.is_empty() {
        // We already found a solution... prune out everything:
        return;
    }

    if i >= g.nodes_id.len() {
        sol.clone_from(&g.nodes_id);
        return;
    }

    for opt in node_options.iter() {
        g.nodes_id[i] = *opt;
        let res = preds.eval(g);
        if !res {continue;}

        plan(g, i + 1, node_options, preds, sol);
        
        if !sol.is_empty() {
            // avoid recalling predicates
            break;
        }
    }

    g.nodes_id[i] = -1;
}

// Simple backtracking algorithm:
fn plan_ordered(g: &mut graph, i: usize, node_order: &Vec<usize>, node_options: &Vec<i32>, preds : &predicates, sol: &mut Vec<i32>) {
    if !sol.is_empty() {
        // We already found a solution... prune out everything:
        return;
    }

    if i >= node_order.len() {
        sol.clone_from(&g.nodes_id);
        return;
    }

    let node = node_order[i];

    for opt in node_options.iter() {
        g.nodes_id[node] = *opt;
        let res = preds.filtered_eval(g, *opt);
        if !res {continue;}

        plan_ordered(g, i + 1, node_order, node_options, preds, sol);
        
        if !sol.is_empty() {
            // avoid recalling predicates
            break;
        }
    }

    // reset for backtrack recursion:
    g.nodes_id[node] = -1;
}

// topo sort and only apply policies
fn topo_sort(g: &graph, v: usize, topo_sorted: &mut Vec<usize>, visited: &mut Vec<bool>) {
    if visited[v] {return;}

    visited[v] = true;
    for ch in g.adjacency_list[v].iter() {
        topo_sort(g, *ch, topo_sorted, visited);
    }

    topo_sorted.push(v)
}


fn test_plan_simple(g: &mut graph) {
    let node_options = vec![0, 1, 2, 3, 4, 5, 6];
    let preds = generate_mock_ud_predicates();

    let mut sol: Vec<i32> = Vec::new();
    plan(g, 0, &node_options, &preds, &mut sol);

    if sol.is_empty() {
        println!("No plans worked...")
    } else {
        println!("Found solution!");
        for i in sol.iter() {
            print!("{} ", i);
        }
        println!();
    }
}

fn test_plan_topo(g: &mut graph) {
    let node_options = vec![0, 1, 2, 3, 4, 5, 6];
    let preds = generate_mock_ud_predicates();

    let mut visited: Vec<bool> = vec![false; g.nodes_id.len()];
    let mut topo_sorted: Vec<usize> = Vec::new();

    for i in 0..g.nodes_id.len() {
        topo_sort(g, i, &mut topo_sorted, &mut visited);
    }

    println!("Topological Sort: ");
    for v in topo_sorted.iter() {
        print!("{} ", v);
    }
    println!();


    let node_options = vec![0, 1, 2, 3, 4, 5, 6];
    let preds = generate_mock_ui_predicates();

    let mut sol: Vec<i32> = Vec::new();
    plan_ordered(g, 0, &topo_sorted, &node_options, &preds, &mut sol);

    if sol.is_empty() {
        println!("No plans worked...")
    } else {
        println!("Found solution!");
        for i in sol.iter() {
            print!("{} ", i);
        }
        println!();
    }

}


fn main() {
    
    let mut g = graph::new(4);
    g.add_edge(0, 1);
    g.add_edge(0, 2);
    g.add_edge(1, 3);
    g.add_edge(2, 3);

    g.print();

    test_plan_simple(&mut g);
    test_plan_topo(&mut g);
}
