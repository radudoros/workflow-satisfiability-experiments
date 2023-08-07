
pub struct PartitionsGenerator {
    pub a: Vec<usize>,
    pub b: Vec<usize>,
    pub m: usize,
    pub n: usize,
    pub done: bool,
}

impl PartitionsGenerator {
    pub fn new(n: usize) -> Self {
        PartitionsGenerator {
            a: vec![0; n],
            b: vec![1; n],
            m: 1,
            n,
            done: false,
        }
    }

    // Generate next partition, given the current state
    // Implementation based on Knuth's book chapter on generating Set Partitions
    pub fn next(&mut self) -> Option<&[usize]> {
        if self.n <= 1 || self.done {
            return None;
        }

        if self.a[self.n-1] != self.m {
            // H3. [Increase an.]
            self.a[self.n-1] += 1;
        } else {
            // H4. [Find j.]
            let mut j = self.n - 2;
            while j != 0 && self.a[j] == self.b[j+1] {
                j -= 1;
            }

            // H5. [Increase aj.]
            if j == 0 {
                self.done = true;
                return None;
            } else {
                self.a[j] += 1;

                // H6. [Zero out aj+1 ... an.]
                self.m = self.b[j+1] + if self.a[j] == self.b[j+1] { 1 } else { 0 };
                j += 1;
                while j < self.n - 1 {
                    self.a[j] = 0;
                    self.b[j+1] = self.m;
                    j += 1;
                }
                self.a[self.n-1] = 0;
            }
        }

        Some(&self.a[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partitions_generator3() {
        let mut pg = PartitionsGenerator::new(3);

        // [0, 0, 0] is default generated
        assert_eq!(pg.next(), Some(&[0, 0, 1][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 0][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 1][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 2][..]));
        assert_eq!(pg.next(), None);
    }

    #[test]
    fn test_partitions_generator4() {
        let mut pg = PartitionsGenerator::new(4);

        // [0, 0, 0, 0] is default generated
        assert_eq!(pg.next(), Some(&[0, 0, 0, 1][..]));
        assert_eq!(pg.next(), Some(&[0, 0, 1, 0][..]));
        assert_eq!(pg.next(), Some(&[0, 0, 1, 1][..]));
        assert_eq!(pg.next(), Some(&[0, 0, 1, 2][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 0, 0][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 0, 1][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 0, 2][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 1, 0][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 1, 1][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 1, 2][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 2, 0][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 2, 1][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 2, 2][..]));
        assert_eq!(pg.next(), Some(&[0, 1, 2, 3][..]));
        assert_eq!(pg.next(), None);
    }

    #[test]
    fn test_partitions_generator_bell() {
        let bell_numbers = [1, 1, 2, 5, 15, 52, 203, 877, 4140, 21147, 115975, 678570, 4213597, 27644437, 190899322]; // and so on, for larger tests

        for n in 0..bell_numbers.len() {
            let mut pg = PartitionsGenerator::new(n);
            let mut partition_count = 1;
    
            while let Some(_) = pg.next() {
                partition_count += 1;
            }
    
            assert_eq!(partition_count, bell_numbers[n]);
        }
    }
}