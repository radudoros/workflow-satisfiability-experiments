pub struct BipartiteGraph<'a> {
    adj: &'a Vec<Vec<usize>>,
    mt: Vec<Option<usize>>,
    used: Vec<bool>,
    m: usize,
    n: usize,
}

impl<'a> BipartiteGraph<'a> {
    pub fn new(adj: &'a Vec<Vec<usize>>, m: usize, n: usize) -> Self {
        BipartiteGraph {
            adj,
            mt: vec![None; n],
            used: vec![false; m],
            m,
            n,
        }
    }

    fn try_kuhn(&mut self, v: usize) -> bool {
        if self.used[v] {
            return false;
        }
        self.used[v] = true;
        for &to in &self.adj[v] {
            if self.mt[to].is_none() || self.try_kuhn(self.mt[to].unwrap()) {
                self.mt[to] = Some(v);
                return true;
            }
        }
        false
    }

    pub fn max_matching_set(&mut self) -> Vec<(usize, usize)> {
        for v in 0..self.m {
            self.used = vec![false; self.m];
            self.try_kuhn(v);
        }

        let mut result = Vec::new();
        for i in 0..self.n {
            if let Some(v) = self.mt[i] {
                result.push((v, i));
            }
        }
        result
    }
    
    pub fn max_matching(&mut self) -> usize {
        let mut count = 0;
        for v in 0..self.m {
            self.used = vec![false; self.m];
            if self.try_kuhn(v) {
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_matching() {
        let adj = vec![
            vec![0],
            vec![1],
            vec![2],
        ];
        let alen = 3;
        let blen = 3;
        let mut bg = BipartiteGraph::new(&adj, alen, blen);
        assert_eq!(bg.max_matching(), 3);
    }

    #[test]
    fn test_disconnected_node() {
        let adj = vec![
            vec![1],
            vec![2],
            vec![],
        ];
        let alen = 3;
        let blen = 3;
        let mut bg = BipartiteGraph::new(&adj, alen, blen);
        assert_eq!(bg.max_matching(), 2);
    }

    #[test]
    fn test_multiple_edges() {
        let adj = vec![
            vec![1, 2],
            vec![0, 2],
            vec![0, 1],
        ];
        let alen = 3;
        let blen = 3;
        let mut bg = BipartiteGraph::new(&adj, alen, blen);
        assert_eq!(bg.max_matching(), 3);
    }

    #[test]
    fn test_no_edges() {
        let adj: Vec<Vec<usize>> = vec![
            vec![],
            vec![],
            vec![],
        ];
        let alen = 3;
        let blen = 3;
        let mut bg = BipartiteGraph::new(&adj, alen, blen);
        assert_eq!(bg.max_matching(), 0);
    }

    #[test]
    fn test_large_matching() {
        let adj = vec![
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
        ];
        let alen = 3;
        let blen = 9;
        let mut bg = BipartiteGraph::new(&adj, alen, blen);
        assert_eq!(bg.max_matching(), 3);
    }

    #[test]
    fn test_large_incomplete_matching() {
        let adj = vec![
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6],
            vec![7],
            vec![8, 9, 10],
            vec![11, 12, 13],
            vec![1],
            vec![14, 15, 16],
            vec![17, 18],
            vec![1],
        ];
        let alen = 10;
        let blen = 19;
        let mut bg = BipartiteGraph::new(&adj, alen, blen);
        assert_eq!(bg.max_matching(), 9);
    }

    // More tests...
}