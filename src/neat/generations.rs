use super::structs::gene::Gene;
use super::structs::genome::Genome;
use super::structs::problem::Problem;
use rand::Rng;
use super::evaluation::evaluate;
use super::visualisation::visualise;
use super::genetics::{cross, mutate, difference, is_valid};

const NUMBER_OF_SPECIMENS:usize = 1000;
const TOURNAMENT_SIZE:usize = 20;
const STAGNATION_TO_EXTINCTION:usize = 20;
const MIXED_CHILDREN_PART:f64 = 0.2;
const PERISHED_PART:f64 = 0.7;
const ITERATIONS:u64 = 10;

fn start_genome(problem:&Problem) -> Genome {
    let mut genes = Vec::new();
    for i in 0..problem.inputs {
        for o in 0..problem.outputs {
            genes.push(Gene::new(
                i,
                // first nodes are for inputs and latter nodes are for outputs 
                problem.inputs+o)
            );
        }
    }
    return  Genome {
        nodes: problem.inputs+problem.outputs,
        genes: genes,
        fitness_complexity_speciation: 0f64,
        fitness : 0f64,
        fitness_complexity: 0f64,
        validation_fitness: 0f64,
        active_nodes: 0
    };
}

fn calculate_base_fitness(problem:&Problem, genome:&mut Genome, dataset:&Dataset) -> f64 {
    let mut result = 0f64;
    for (input, output) in dataset.inputs.iter().zip(dataset.outputs.iter()) {
        // first value is bias 
        let mut network_input = vec![1f64];
        network_input.extend(input.iter());

        let evaluation_result = evaluate(
            &input,
            &problem,
            genome,
        );
        let mut fitness = 0f64;
        for (result, expected) in evaluation_result.iter().zip(output) {
            fitness += f64::abs(result - expected);
        }
        fitness /= evaluation_result.len() as f64;
        result += fitness;
    }
    result /= dataset.inputs.len() as f64;
    
    return 1f64 - result;
}

fn calculate_fitness(problem:&Problem, genome:&mut Genome, species_size: usize, training_dataset:&Dataset) {
    genome.fitness = calculate_base_fitness(problem, genome, training_dataset);
    genome.fitness_complexity = genome.fitness;
    genome.fitness_complexity /= (genome.active_genes() / 3  + 1) as f64;
    genome.fitness_complexity /= (genome.active_nodes / 20 + 1) as f64;
    genome.fitness_complexity_speciation = genome.fitness_complexity;
    genome.fitness_complexity_speciation /= species_size as f64;
}

pub fn tournament<'a, R: Rng + ?Sized>(rng: &mut R, specimens: &'a Vec<Genome>) -> &'a Genome {
    let mut best = &specimens[rng.gen_range(0..specimens.len())]; 
    for _ in 0..TOURNAMENT_SIZE {
        let contestant = &specimens[rng.gen_range(0..specimens.len())]; 
        if contestant.fitness_complexity_speciation > best.fitness_complexity_speciation {
            best = contestant;
        }
    }
    return best;
}

pub struct Specie {
    pub representative: Genome,
    pub specimens: Vec<Genome>,
    pub best_fitness: f64
    // calculate number of children using best fitness 
}

pub type DataFrame = std::vec::Vec<std::vec::Vec<f64>>;

#[derive(Debug)]
pub struct Dataset {
    pub inputs: DataFrame,
    pub outputs: DataFrame
}

// struct used to reduce number of passed arguments 
// between functions in this module
pub struct Generations<'a, R: Rng + ?Sized> {
    random: &'a mut R,
    problem:&'a Problem,
    training_dataset: &'a Dataset,
    validation_dataset: &'a Dataset,
    species : Vec<Specie>,
    best: Genome,
    fitness_stagnant: usize,
    iteration: u64,

    children : Vec<Genome>,
    species_fitness_sum: f64,
    iteration_best:Genome
}

pub fn run<R: Rng + ?Sized>(mut rng: &mut R, training_dataset: &Dataset, validation_dataset: &Dataset) -> Genome {
    
    // +1 for the bias 
    let problem = Problem {inputs:training_dataset.inputs[0].len()+1, outputs:training_dataset.outputs[0].len()};
    println!("{:?}", problem);
    
    // create one start specie 
    let species : Vec<Specie> = vec![Specie {
        representative: start_genome(&problem),
        specimens: (0..NUMBER_OF_SPECIMENS).map(|_| {
            let mut genome = start_genome(&problem);
            // initial mutation 
            mutate(&mut rng, &mut genome, &problem, 0);
            return genome;
        }).collect(),
        best_fitness:0.
    }];

    let mut _self = Generations {
        best:species[0].representative.clone(), 
        iteration_best:species[0].representative.clone(), 
        species,
        fitness_stagnant:0,
        children: vec![],
        problem:&problem,random:rng,
        training_dataset,
        validation_dataset,
        iteration: 0,
        species_fitness_sum: 0f64
    };

    for iteration in 1..ITERATIONS {
        _self.iteration = iteration;
        _self.iteration();
    }

    println!("[e]\tbest fitness c: {:.5}, \tbest fitness: {:.5}, \nnodes: {:.5}", 
        _self.best.fitness_complexity, _self.best.validation_fitness, _self.best.active_nodes);

    visualise(&_self.best, &_self.problem);
    return _self.best;
}

impl<R: Rng + ?Sized> Generations<'_, R> {

fn iteration(&mut self) {
    self.children.clear();
    self.iteration_best = self.species[0].representative.clone();

    // [Extinction to two specimen due to stagnation]
    if self.fitness_stagnant > STAGNATION_TO_EXTINCTION {
        self.great_extinction(self.iteration);
    } else {
        self.species_fitness_sum = 0.;

        self.evaluate_species();    
        self.eliminate_and_reproduce();
        self.mixed_children();

        for specie in self.species.iter_mut() {
            specie.specimens.clear();
        }
    }
    self.assign_children_to_species();

    self.species.retain(|s| !s.specimens.is_empty());

    // speciation
    println!(
        "[{}]\tbest fitness c: {:.5}, \tbest fitness: {:.5}, \tnodes: {:.5}, \tspecies: {:.5}", 
        self.iteration, 
        self.iteration_best.fitness_complexity, self.iteration_best.validation_fitness, 
        self.iteration_best.active_nodes, self.species.len());

    self.fitness_stagnant += 1;
}

fn eliminate_and_reproduce(&mut self) {
     // [Eliminate and reproduce speciment]
     for specie in self.species.iter_mut() {
        let specimens = &mut specie.specimens;
        
        // [Eliminate the weakest specimens]
        specimens.sort_by(|a, b| {b.fitness_complexity
            .partial_cmp(&a.fitness_complexity).unwrap()});

        if specimens.len() > 2  {
            let to_remove = ((specimens.len() - 2) as f64 * PERISHED_PART) as usize;
            for _ in 0..to_remove {
                specimens.remove(specimens.len()  - 1);
            }
        }
        
        // [Create children of the specie]
        let children_count = (
            (NUMBER_OF_SPECIMENS as f64 * (1. - MIXED_CHILDREN_PART) 
            * specie.best_fitness / self.species_fitness_sum) as f64) as usize;
        
        for _ in 0..children_count {
            // choose parents
            let parent_a = tournament(&mut self.random, &specimens);
            let parent_b = tournament(&mut self.random, &specimens);

            // create child 
            let mut new_child = cross(&mut self.random, parent_a, parent_b);
            // mutate it  
            mutate(&mut self.random, &mut new_child, self.problem, self.iteration);
            
            if is_valid(&new_child, self.problem) {
                self.children.push(new_child);
            }
        }
        // change representative to random specimen
        specie.representative = specie.specimens[self.random.gen_range(0..specie.specimens.len())].clone();
    }
}

fn evaluate_species(&mut self) { 
    let species_len = self.species.len();
    for (index, specie) in self.species.iter_mut().enumerate() {
        let specimens = &mut specie.specimens;
        let mut specie_best = specie.representative.clone();
        let specimens_len = specimens.len();
        for specimen in specimens.iter_mut() {
            calculate_fitness(self.problem, specimen, specimens_len, self.training_dataset);
            specimen.validation_fitness = calculate_base_fitness(self.problem, specimen, self.validation_dataset);
            if specimen.validation_fitness > self.best.validation_fitness {
                self.best = specimen.clone();
                self.fitness_stagnant = 0;
                self.children.push(specimen.clone());
            }
            if specimen.validation_fitness > specie_best.validation_fitness {
                specie_best = specimen.clone();
            }
            if specimen.validation_fitness > self.iteration_best.validation_fitness {
                self.iteration_best = specimen.clone();
            }
        }
                
        println!("[{}/{}]\tbest fitness c: {:.5}, \tbest fitness: {:.5}, \tnodes: {:.5}, \tspecies: {:.5}",  
        self.iteration, index, 
        specie_best.fitness_complexity, specie_best.validation_fitness, 
        specie_best.active_nodes, species_len);

        specie.best_fitness = specie_best.validation_fitness;
        self.species_fitness_sum+=specie_best.validation_fitness;
    }
}

fn mixed_children(&mut self) {
    // choose two specimen from two random species in tournament and cross them]
    for _ in 0..(NUMBER_OF_SPECIMENS as f64 * MIXED_CHILDREN_PART) as usize {
    // choose parents
    let population_a = &self.species[self.random.gen_range(0..self.species.len())].specimens;
    let population_b = &self.species[self.random.gen_range(0..self.species.len())].specimens;
    let parent_a = tournament(&mut self.random, population_a);
    let parent_b = tournament(&mut self.random, population_b);

    // create child 
    let mut new_child = cross(&mut self.random, parent_a, parent_b);
    // mutate it  
    mutate(&mut self.random, &mut new_child, self.problem, self.iteration);
    
    if is_valid(&new_child, self.problem) {
        self.children.push(new_child);
    }
}
}

fn assign_children_to_species(&mut self) {
    for child in self.children.iter() {
        let mut asssigned = false;
        for specie in self.species.iter_mut() {
            if difference (child, &specie.representative) < 1.0 {
                specie.specimens.push(child.clone());
                asssigned = true;
                break;
            }
        }
        // create new specie for the child if it doesn't match any
        if !asssigned {
            self.species.push(Specie {
                representative: child.clone(),
                specimens: vec![child.clone(), child.clone()],
                best_fitness:0.
            })
        }
    }
}

fn great_extinction(&mut self, iteration: u64) {
    println!("Noah get the boat");
    let mut specimens:Vec<Genome> = self.species.iter().map(|s| s.specimens.clone()).flatten().collect();
    specimens.sort_by(|a, b| {b.fitness_complexity.partial_cmp(&a.fitness_complexity).unwrap()});
    
    // choose parents
    let parent_a = &specimens[0];
    let parent_b = &specimens[1];

    for _ in 0..NUMBER_OF_SPECIMENS {
        // create child 
        let mut new_child;
        loop {
            new_child = cross(&mut self.random, parent_a, parent_b);
            // mutate it  
            mutate(&mut self.random, &mut new_child, self.problem, iteration);
            
            if is_valid(&new_child, self.problem) {
                self.children.push(new_child);
                break;
            }
        }
    }
    self.species = vec![Specie {
        representative: parent_a.clone(),
        specimens: Vec::new(),
        best_fitness:0.
    }, Specie {
        representative: parent_b.clone(),
        specimens: Vec::new(),
        best_fitness:0.
    }
    ];
    self.fitness_stagnant=0;
}
}