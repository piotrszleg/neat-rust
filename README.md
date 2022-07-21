# Rust implementation of NEAT

Rust implementation of Neuro Evolution of Augmenting Topologies ([original paper](http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)).

Features:
- parametrization
- loading custom csv dataset
- visualization using Cytoscape.js

![network sample](https://user-images.githubusercontent.com/16499460/180267861-15025679-884a-4600-bd61-c036c853e4c2.png)

## Running

You'll need Rust and Cargo on your machine.

Go to the cloned repository folder and input the following command into your terminal:

```bash
cargo run
```

You can change `inputs.csv` and `outputs.csv` files to run the network on different datasets.

They should contain comma separated values and the first row should contain column names. See the sample files in datasets folder for reference.

## Configuration

Files called `generations.rs` and `genetics.rs` contain constants that you can change in order to parametrize the algorithm.
