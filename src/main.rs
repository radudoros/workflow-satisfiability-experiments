use predicates::{binary_predicates, generate_mock_ud_predicates, generate_mock_ui_predicates};
use workflow::{graph, topo_sort};

mod workflow;
mod predicates;

// Simple backtracking algorithm:
fn plan(g: &mut graph, i: usize, node_options: &Vec<i32>, preds : &binary_predicates, sol: &mut Vec<i32>) {
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

// Backtracking while respecting the topological order:
fn plan_ordered(g: &mut graph, i: usize, node_order: &Vec<usize>, node_options: &Vec<i32>, preds : &binary_predicates, sol: &mut Vec<i32>) {
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
    let topo_sorted = topo_sort(g);

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
