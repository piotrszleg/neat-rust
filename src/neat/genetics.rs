use rand::Rng;
use super::structs::genome::Genome;
use super::structs::problem::Problem;
use super::structs::gene::Gene;
use std::cmp::max;

const MUTATE_WEIGHT_CHANCE:f64 = 0.15;
const WEIGHT_CHANGE:f64 = 0.25;
const INSERT_NODE_CHANCE:f64 = 0.05;
const INSERT_CONNECTION_CHANCE:f64 = 0.05;

const DISABLE_CONNECTION_CHANCE:f64 = 0f64;

fn choice<R: Rng + ?Sized>(rng: &mut R, chance:f64) -> bool {
    return rng.gen_range(0f64..1f64) < chance;
}

fn max_edges_in_undirected_graph(n: u64) -> u64 {
    // assuming there are no self loops and no double edges 
    return (n*(n-1))/2;
}

pub fn mutate<R: Rng + ?Sized>(mut rng: &mut R, mut genome:&mut Genome, problem:&Problem, innovation:u64) {
    let mut genes_to_add:Vec<Gene> = Vec::new();
    for gene in genome.genes.iter_mut() {
        // if > mutate weight chance 
        if choice(rng, MUTATE_WEIGHT_CHANCE) {
            gene.weight = gene.weight + rng.gen_range(-WEIGHT_CHANGE..WEIGHT_CHANGE);
        }
        if choice(rng, DISABLE_CONNECTION_CHANCE) {
            gene.enabled = false;
        }
        if choice(rng, INSERT_NODE_CHANCE) {
            gene.enabled = false;
            genome.nodes+=1;
            // add two connections to and out of the new node 
            genes_to_add.push(
                Gene {
                    input:gene.input,
                    output: genome.nodes-1,
                    weight: 1f64,
                    enabled: true,
                    innovation: innovation
                }
            );
            genes_to_add.push(
                Gene {
                    input: genome.nodes-1,
                    output: gene.output,
                    weight: gene.weight,
                    enabled: true,
                    innovation: innovation
                }
            );
            genes_to_add.push(
                Gene {
                    input: 0,
                    output: genome.nodes-1,
                    weight: rng.gen_range(-0.1..0.1),
                    enabled: true,
                    innovation: innovation
                }
            );
        }
    }
    genome.genes.append(&mut genes_to_add);
    
    if choice(rng, INSERT_CONNECTION_CHANCE) {
        insert_connection(&mut rng, &mut genome, &problem, innovation);
    }
}

pub fn is_valid(genome:&Genome, problem:&Problem) -> bool {
    return !genome.has_cycles() && genome.genes.iter().fold(true, |acc, gene| acc && gene.output >= problem.inputs);
}

fn insert_connection<R: Rng + ?Sized>(rng: &mut R, genome:&mut Genome, problem:&Problem, innovation:u64) {
    // Lazy generate all options and choose stopping point

    let mut possible_connections = max_edges_in_undirected_graph(genome.nodes as u64) as i64;
    // subtract connections from each node to inputs and itself 
    possible_connections -= ((problem.inputs + 1) * genome.nodes) as i64;
    // subtract already existing connections 
    possible_connections -= genome.genes.len() as i64;

    if possible_connections > 0 {
        let new_connection = rng.gen_range(0..possible_connections);
        
        // iterate over possible connections, increment if connection doesn't exist 
        // if index == new_path add connection 
        let mut index = 0;
        for input in 0..genome.nodes-1 {
            // output can't lead to input node
            for output in problem.inputs..genome.nodes {
                if input != output && !genome.has_connection(input, output) {   
                    if index == new_connection { 
                        genome.genes.push(Gene{
                            input, output,
                            weight: 0.0,
                            enabled: true,
                            innovation:innovation
                        });
                        if genome.has_cycles() {
                            // graph created by the genome is cyclical 
                            // but we don't want to create this connection later again 
                            // so the best option is to simply disable it 
                            let last_gene = genome.genes.len()-1;
                            genome.genes[last_gene].enabled = false;
                        }
                    }
                    index += 1;
                }
            }
        }
    }
}

pub fn cross<R: Rng + ?Sized>(rng: &mut R, genome_a:&Genome, genome_b:&Genome) -> Genome {
    let mut a_index = 0usize;
    let mut b_index = 0usize;
    let mut a_last_copy = 0usize;
    let mut b_last_copy = 0usize;
    let mut genome = Genome {
        nodes : max(genome_a.nodes, genome_b.nodes),
        genes : Vec::new(),
        fitness : 0f64,
        fitness_complexity: 0f64,
        fitness_complexity_speciation : 0f64,
        validation_fitness: 0f64,
        active_nodes: 0
    };

    while a_index < genome_a.genes.len() && b_index < genome_b.genes.len() {
        if genome_a.genes[a_index].innovation < genome_b.genes[b_index].innovation {
            a_index += 1;
        } else if genome_a.genes[a_index].innovation > genome_b.genes[b_index].innovation {
            b_index += 1;
        } else {
            // println!("Same innovation at {:?}", a_index);
            // copy previous genes of genome with better fitness 

            // choose genome with better fitness 
            if genome_a.fitness_complexity_speciation > genome_b.fitness_complexity_speciation {
                genome.genes.extend(genome_a.genes[a_last_copy..a_index].iter());
            } else if genome_a.fitness_complexity_speciation < genome_b.fitness_complexity_speciation {
                genome.genes.extend(genome_b.genes[b_last_copy..b_index].iter());
            } 
            // choose at random if genomes have the same fitness
            else if choice(rng, 0.5) {
                genome.genes.extend(genome_a.genes[a_last_copy..a_index].iter());
            } else {
                genome.genes.extend(genome_b.genes[b_last_copy..b_index].iter());
            }
            // TODO: more complicated method from the paper 

            // choose randomly one of the matching genes 
            if choice(rng, 0.5) {
                genome.genes.push(genome_a.genes[a_index]);
            } else {
                genome.genes.push(genome_b.genes[b_index]);
            }
            a_last_copy = a_index + 1 ;
            b_last_copy = b_index + 1 ;
            a_index += 1;
            b_index += 1;
        }
    }

    // excess genes
    if genome_a.genes.len() > genome_b.genes.len() {
        genome.genes.extend(genome_a.genes[a_last_copy..genome_a.genes.len()].iter());
    } else if genome_b.genes.len() > genome_a.genes.len() {
        genome.genes.extend(genome_b.genes[b_last_copy..genome_b.genes.len()].iter());
    }
    // choose genome with better fitness 
    else if genome_a.fitness_complexity_speciation > genome_b.fitness_complexity_speciation {
        genome.genes.extend(genome_a.genes[a_last_copy..genome_a.genes.len()].iter());
    } else if genome_a.fitness_complexity_speciation < genome_b.fitness_complexity_speciation {
        genome.genes.extend(genome_b.genes[b_last_copy..genome_b.genes.len()].iter());
    } 
    // choose at random if genomes have the same fitness
    else if choice(rng, 0.5) {
        genome.genes.extend(genome_a.genes[a_last_copy..genome_a.genes.len()].iter());
    } else {
        genome.genes.extend(genome_b.genes[b_last_copy..genome_b.genes.len()].iter());
    }

    return genome;
}

const C1:f64 = 1.5;
const C2:f64 = 1.5;
const C3:f64 = 0.5;

pub fn difference (genome_a:&Genome, genome_b:&Genome) -> f64 {
    let mut a_index = 0usize;
    let mut b_index = 0usize;
    let mut disjoint = 0;
    let mut matching = 0;
    let mut weight_difference = 0f64;

    while a_index < genome_a.genes.len() && b_index < genome_b.genes.len() {
        if genome_a.genes[a_index].innovation < genome_b.genes[b_index].innovation {
            a_index += 1;
            disjoint += 1;
        } else if genome_a.genes[a_index].innovation > genome_b.genes[b_index].innovation {
            b_index += 1;
            disjoint += 1;
        } else {
            weight_difference += f64::abs(genome_a.genes[a_index].weight - genome_b.genes[b_index].weight);
            a_index += 1;
            b_index += 1;
            matching += 1;
        }
    }
    let n = usize::max(genome_a.genes.len(), genome_b.genes.len());
    let w = weight_difference / matching as f64;
    let excess =  genome_a.genes.len() as i64 - 1 - a_index as i64 
                + genome_b.genes.len() as i64 - 1 - b_index as i64; 

    return C1 * (excess as f64) / (n as f64) + C2 * (disjoint as f64) / (n as f64) + C3 * w;
}