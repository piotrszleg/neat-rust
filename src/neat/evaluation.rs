use std::collections::HashSet;
use std::f64::consts::E;
use super::structs::problem::Problem;
use super::structs::genome::Genome;

const LOG:bool = false;

#[derive(Debug)]
#[derive(Clone)]
struct Node {
    expected_inputs: usize,
    value: f64,
    propagated: bool,
    is_input: bool,
}

fn sigmoid(value: f64) -> f64 {
    return 1f64 / (1f64 + E.powf(-value));
}

// easiest: 
// count inputs and forward only if all are satisfied (cycles can't count) 
// but also: hand written recursion limit
fn evaluate_recursive(genome:&Genome, nodes:&mut Vec<Node>, nodes_to_process:HashSet<usize>) {
    if LOG {
        println!("evaluate_recursive {:?}", nodes_to_process);
    }
    let mut new_nodes_to_process = HashSet::new();
    for processed_node in nodes_to_process {
        for gene in genome.genes.iter() {
            if gene.input == processed_node && gene.enabled { // enabled here prevents from infinite recursion 
                let mut value = nodes[processed_node].value;
                // don't add sigmoid to input nodes to not distort the input range 
                if !nodes[processed_node].is_input {
                    value = sigmoid(value);
                }
                nodes[gene.output].value += value * gene.weight;
                nodes[gene.output].expected_inputs -= 1;
                if nodes[gene.output].expected_inputs <= 0 && !nodes[gene.output].propagated {
                    nodes[gene.output].propagated = true;
                    new_nodes_to_process.insert(gene.output);
                }
            }
        }
    }

    if !new_nodes_to_process.is_empty() {
        evaluate_recursive(genome, nodes, new_nodes_to_process);
    }
}

pub fn evaluate(input:&[f64], problem:&Problem, genome:&mut Genome) -> Vec<f64> {
    let mut nodes = vec![Node {expected_inputs:0, value:0f64, propagated:false, is_input:false}; genome.nodes];
    let mut nodes_to_process = HashSet::new();

    for gene in genome.genes.iter() {
        if gene.enabled {
            nodes[gene.output].expected_inputs += 1;
        }
    }

    for (i, x) in input.iter().enumerate() {
        nodes[i].is_input = true;
        nodes[i].value = *x;
        nodes[i].propagated = true;
        nodes_to_process.insert(i);
    }

    evaluate_recursive(genome, &mut nodes, nodes_to_process);
    
    /*
    for (i, x) in nodes.iter().enumerate() {
        if !x.propagated {
            println!("Not propagated node: [{:?}] {:?}", i, x);
        }
    }
    */
    genome.active_nodes = nodes.iter().filter(|x| x.propagated).count() + problem.outputs;

    return nodes[problem.inputs..problem.inputs+problem.outputs]
           .into_iter().map(|node| sigmoid(node.value)).collect();
}