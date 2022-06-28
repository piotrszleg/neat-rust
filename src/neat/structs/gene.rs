#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
pub struct Gene {
    pub input: usize,
    pub output: usize,
    pub weight: f64,
    pub enabled: bool,
    pub innovation: u64
}

impl Gene {
    pub fn new(input: usize, output: usize) -> Gene {
        return Gene{
            input: input, 
            output: output, 
            weight: 1f64,
            enabled: true,
            innovation: 0
        };
    }
}