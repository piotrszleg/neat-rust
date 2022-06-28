use rand::thread_rng;
use csv;
mod neat;

// visualization
// https://js.cytoscape.org/#notation/elements-json

/*
TODO: 
- verify structure
- load custom dataset 
*/


fn read_data_frame(path:&str) -> neat::generations::DataFrame {
    use std::fs::File;

    println!("Reading {:?}", path);

    let file = File::open(path).unwrap();
    let mut rdr = csv::Reader::from_reader(file);
    let mut result = Vec::new();
    for record in rdr.records() {
        let parsed = record
            .unwrap()
            .iter()
            .map(|x| x.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", parsed);
        result.push(parsed);
    }

    return result;
}

fn main() {

    let mut rng = thread_rng();

    let dataset: neat::generations::Dataset = neat::generations::Dataset {
        inputs: read_data_frame("inputs.csv"),
        outputs: read_data_frame("outputs.csv")
    };

    neat::generations::run(&mut rng, &dataset, &dataset);
}
