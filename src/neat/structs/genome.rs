use super::gene::Gene;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Genome {
    pub nodes: usize,
    pub genes: Vec<Gene>,
    pub fitness: f64,
    pub fitness_complexity: f64,
    pub fitness_complexity_speciation: f64,
    pub active_nodes: usize,
    pub validation_fitness: f64
}

impl Clone for Genome {
    fn clone(&self) -> Self {
        Genome { nodes: self.nodes, genes: self.genes.clone(), 
            fitness_complexity_speciation: self.fitness_complexity_speciation, fitness: self.fitness,
            fitness_complexity: self.fitness_complexity,
            active_nodes:self.active_nodes, validation_fitness:self.validation_fitness }
    }
}

impl Genome {
    pub fn active_genes(&self) -> usize {
        let mut count = 0;
        for gene in self.genes.iter() {
            if gene.enabled && gene.weight != 0f64 {
                count += 1;
            }
        }
        return count;
    }
    pub fn has_connection(&self, input: usize, output: usize) -> bool {
        for gene in self.genes.iter() {
            if (gene.input == input && gene.output == output) 
                && (gene.input == output && gene.output == input) {
                    return true;
            }
        }
        return false;
    }

    fn has_cycles_recursive(&self, vertex:usize, visited:&mut HashSet<usize>, recursion_stack:&mut HashSet<usize>) -> bool {
        visited.insert(vertex);
        recursion_stack.insert(vertex);

        for gene in self.genes.iter() {
            if gene.input == vertex {
                let neighbor = gene.output;
                if !visited.contains(&neighbor) {
                    if self.has_cycles_recursive(neighbor, visited, recursion_stack) {
                        return true;
                    }
                } else if recursion_stack.contains(&neighbor) {
                    return true;
                }
            }
        }
        
        recursion_stack.remove(&vertex);
        return false;
    }

    pub fn has_cycles(&self) -> bool {
        let mut visited: HashSet<usize> = HashSet::new();
        let mut recursion_stack: HashSet<usize> = HashSet::new();

        for node in 0..self.nodes {
            if !visited.contains(&node) {
                if self.has_cycles_recursive(node, &mut visited, &mut recursion_stack) {
                    return true;
                }
            }
        }
        
        return false;
    }
}
