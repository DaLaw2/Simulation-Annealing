# Simulation-Annealing-Implement
This repository contains a rust implementation of the Simulated Annealing (SA) algorithm, a probabilistic technique for approximating the global optimum of a given function. Specifically, this implementation can be applied for solving the Traveling Salesman Problem (TSP).
## Getting Started
### __Prerequisites__
To compile and run this project, the following prerequisites are required:
- __Rust__: The Rust programming language.
- __Cargo__: The Rust package manager.
- Other Rust packages including: `regex`, `rand`, `calamine` and `std`.
### Installation
Clone the repository to your local machine:
```
git clone https://github.com/<username>/<repository>.git
```
Navigate to the project directory:
```
cd simulated-annealing-rust
```
Build the project:
```
cargo build
```
## Usage
The algorithm can be executed from the command line with the following command:
```
cargo run -- --input=<path_to_input_file> --output=<path_to_output_file> --config=<path_to_config_file>
```
- `--input`: Specifies the path to the input Excel file (.xlsx) that contains the 2D coordinates of the points.
- `--output`: Specifies the path to the output text file where the results will be saved.
- `--config`: Specifies the path to the configuration text file that contains the parameters of the algorithm.
## Configuration
The configuration file allows to tune the parameters of the SA algorithm:
- `initial_temperature`: The initial temperature of the system.
- `minimum_temperature`: The minimum temperature of the system.
- `temperature_decay`: The rate at which the temperature decays.
- `max_iterations`: The maximum number of iterations.
- `generation_method`: The method to generate the next solution. This can be `Swap` or `Reverse`.
- `cooling_method`: The method for cooling. This can be `ExponentialMultiplicativeCooling`, `LogarithmicMultiplicativeCooling`, `LinearMultiplicativeCooling`, or `QuadraticMultiplicativeCooling`.
## Output
The program will produce an output text file with the following information:
- The current iteration count.
- The change in energy (ΔE).
- The current temperature (T).
- The probability of acceptance of a new solution.
- The final solution (a sequence of points representing a possible solution for the TSP).
- The total path length of the solution.
- The total execution time.
