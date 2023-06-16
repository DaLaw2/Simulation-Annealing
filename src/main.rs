use std::env;
use regex::Regex;
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use calamine::{Reader, Xlsx, open_workbook};

struct ArgKind {
    input: Option<String>,
    output: Option<String>,
    config: Option<String>,
}

struct ConfigKind {
    initial_temperature: f64,
    minimum_temperature: f64,
    temperature_decay: f64,
    max_iterations: usize,
    generation_method: GenerationMethod,
    cooling_method: CoolingMethod,
}

enum GenerationMethod {
    Swap,
    Insert,
    Reverse,
    Kopt(usize),
}

enum CoolingMethod {
    ExponentialMultiplicativeCooling,
    LogarithmicMultiplicativeCooling,
    LinearMultiplicativeCooling,
    QuadraticMultiplicativeCooling,
}

fn get_args() -> ArgKind {
    let mut args = ArgKind {
        input: None,
        output: None,
        config: None,
    };
    let command_line: Vec<String> = env::args().collect();
    for arg in &command_line[1..] {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() != 2 {
            panic!("Invalid argument.");
        }
        match parts[0] {
            "--input" => args.input = Some(parts[1].to_string()),
            "--output" => args.output = Some(parts[1].to_string()),
            "--config" => args.config = Some(parts[1].to_string()),
            _ => panic!("Invalid argument."),
        }
    }
    args
}

fn read_xlsx(input_path: String) -> Vec<Vec<f64>> {
    let mut excel_data: Vec<Vec<f64>> = Vec::new();
    let mut excel_file: Xlsx<_> = open_workbook(input_path).expect("Cannot open file.");
    let sheet_name = excel_file.sheet_names().get(0).expect("No sheet found.").clone();
    if let Some(Ok(sheet)) = excel_file.worksheet_range(sheet_name.as_str()) {
        for row in sheet.rows() {
            let mut row_data: Vec<f64> = Vec::new();
            for col in row.iter() {
                let col_data = match col {
                    calamine::DataType::Int(i) => *i as f64,
                    calamine::DataType::Float(f) => *f,
                    _ => panic!("Invalid value in data sheet."),
                };
                row_data.push(col_data);
            }
            excel_data.push(row_data);
        }
    }
    excel_data
}

fn read_config(config_path: String) -> ConfigKind {
    let mut config = ConfigKind {
        initial_temperature: 0.0,
        minimum_temperature: 0.0,
        temperature_decay: 0.0,
        max_iterations: 0,
        generation_method: GenerationMethod::Swap,
        cooling_method: CoolingMethod::ExponentialMultiplicativeCooling,
    };
    let config_file = File::open(config_path).expect("Fail read config file");
    let reader = BufReader::new(config_file);
    for line in reader.lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split('=').map(|part| part.trim()).collect();
            if parts.len() == 2 {
                let key = parts[0];
                let value = parts[1];
                match key {
                    "initial_temperature" => config.initial_temperature = value.parse::<f64>().expect("Wrong configuration."),
                    "minimum_temperature" => config.minimum_temperature = value.parse::<f64>().expect("Wrong configuration."),
                    "temperature_decay" => config.temperature_decay = value.parse::<f64>().expect("Wrong configuration."),
                    "max_iterations" => config.max_iterations = value.parse::<usize>().expect("Wrong configuration."),
                    "generation_method" => config.generation_method = match value {
                        "Swap" => GenerationMethod::Swap,
                        "Insert" => GenerationMethod::Insert,
                        "Reverse" => GenerationMethod::Reverse,
                        _ => {
                            let regex = Regex::new(r"(\d+)-opt").unwrap();
                            if let Some(captures) = regex.captures(value) {
                                if let Some(k) = captures.get(1) {
                                    let k_value = k.as_str().parse::<usize>().expect("Wrong configuration.");
                                    GenerationMethod::Kopt(k_value)
                                } else {
                                    panic!("Unknown configuration.");
                                }
                            } else {
                                panic!("Unknown configuration.");
                            }
                        },
                    },
                    "cooling_method" => config.cooling_method = match value {
                        "ExponentialMultiplicativeCooling" => CoolingMethod::ExponentialMultiplicativeCooling,
                        "LogarithmicMultiplicativeCooling" => CoolingMethod::LogarithmicMultiplicativeCooling,
                        "LinearMultiplicativeCooling" => CoolingMethod::LinearMultiplicativeCooling,
                        "QuadraticMultiplicativeCooling" => CoolingMethod::QuadraticMultiplicativeCooling,
                        _ => panic!("Unknown configuration."),
                    },
                    _ => panic!("Unknown configuration."),
                }
            }
        }
    }
    config
}

fn euclidean_distance(point1: &Vec<f64>, point2: &Vec<f64>) -> f64 {
    if point1.len() != point2.len() {
        panic!("Invalid data sheet.");
    }
    let mut distance = 0.0;
    for dimension in 0..point1.len() {
        distance += (point1[dimension] - point2[dimension]).powf(2.0);
    }
    distance.sqrt()
}

fn calc_points_distance(points: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut adjacency_matrix: Vec<Vec<f64>> = vec![vec![0.0; points.len()]; points.len()];
    for i in 0..points.len() {
        for j in (i+1)..points.len() {
            if i == j {
                continue
            }
            let distance = euclidean_distance(&points[i], &points[j]);
            adjacency_matrix[i][j] = distance;
            adjacency_matrix[j][i] = distance;
        }
    }
    adjacency_matrix
}

fn initialize_solution(point_amount: usize) -> Vec<usize> {
    let mut rng = thread_rng();
    let mut solution: Vec<usize> = (0..point_amount).collect();
    solution.shuffle(&mut rng);
    solution
}

fn swap(solution: &Vec<usize>) -> Vec<usize> {
    let mut neighbor = solution.clone();
    let mut rng = rand::thread_rng();
    let (point1, point2) = loop {
        let (i, j) = (rng.gen_range(0..solution.len()), rng.gen_range(0..solution.len()));
        if i == j {
            continue;
        } else {
            break (i, j);
        }
    };
    neighbor.swap(point1, point2);
    neighbor
}

fn insert(solution: &Vec<usize>) -> Vec<usize> {
    //No implementation
    solution.clone()
}

fn reverse (solution: &Vec<usize>) -> Vec<usize> {
    let mut neighbor = solution.clone();
    let mut rng = rand::thread_rng();
    let (mut point1, mut point2) = loop {
        let (i, j) = (rng.gen_range(0..solution.len()), rng.gen_range(0..solution.len()));
        if i == j || (if i > j {i-j} else {j-i}) < 2 {
            continue;
        } else {
            break (i, j);
        }
    };
    if point1 > point2 {
        std::mem::swap(&mut point1, &mut point2);
    }
    neighbor[point1..=point2].reverse();
    neighbor
}

fn k_opt(solution: &Vec<usize>, k: usize) -> Vec<usize> {
    //No implementation
    solution.clone()
}

fn calc_path_length(distance: &Vec<Vec<f64>>, solution: &Vec<usize>) -> f64 {
    let mut length = 0.0;
    for i in 0..(solution.len()-1) {
        length += distance[solution[i]][solution[i+1]];
    }
    length += distance[distance.len()-1][0];
    length
}

fn exponential_multiplicative_cooling(initial_temperature: f64, temperature_decay: f64, iterations: usize) -> f64 {
    initial_temperature * temperature_decay.powf(iterations as f64)
}

fn logarithmic_multiplicative_cooling(initial_temperature: f64, temperature_decay: f64, iterations: usize) -> f64 {
    initial_temperature / (1.0 + temperature_decay * (1.0 + iterations as f64).ln())
}

fn linear_multiplicative_cooling(initial_temperature: f64, temperature_decay: f64, iterations: usize) -> f64 {
    initial_temperature / (1.0 + temperature_decay * iterations as f64)
}

fn quadratic_multiplicative_cooling(initial_temperature: f64, temperature_decay: f64, iterations: usize) -> f64 {
    initial_temperature / (1.0 + temperature_decay * iterations.pow(2) as f64)
}

fn simulation_annealing(points: &Vec<Vec<f64>>, distance: &Vec<Vec<f64>>, config: &ConfigKind, output_message: &mut String) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut solution = initialize_solution(points.len());
    let initial_temperature = config.initial_temperature;
    let mut temperature = config.initial_temperature;
    let minimum_temperature = config.minimum_temperature;
    let temperature_decay = config.temperature_decay;
    let mut iterations = 1_usize;
    let max_iterations = config.max_iterations;
    let generation_method = &config.generation_method;
    let cooling_method = &config.cooling_method;
    while temperature > minimum_temperature && iterations < max_iterations {
        let neighbor = match generation_method {
            GenerationMethod::Swap => swap(&solution),
            GenerationMethod::Insert => insert(&solution),
            GenerationMethod::Reverse => reverse(&solution),
            GenerationMethod::Kopt(k) => k_opt(&solution, *k),
        };
        let solution_length = calc_path_length(&distance, &solution);
        let neighbor_length = calc_path_length(&distance, &neighbor);
        let random_number: f64 = rng.gen();
        let accept_probability = std::f64::consts::E.powf((solution_length - neighbor_length) / temperature);
        output_message.push_str(&format!("{} times iterations.\n", iterations));
        output_message.push_str(&format!("ΔE={}, T={}\n", neighbor_length - solution_length, temperature));
        output_message.push_str(&format!("Probability of accept = {}\n", accept_probability));
        if accept_probability > random_number {
            solution = neighbor;
            output_message.push_str("Accept new solution.\n\n");
        } else {
            output_message.push_str("Reject new solution.\n\n");
        }
        temperature = match cooling_method {
            CoolingMethod::ExponentialMultiplicativeCooling => exponential_multiplicative_cooling(initial_temperature, temperature_decay, iterations),
            CoolingMethod::LogarithmicMultiplicativeCooling => logarithmic_multiplicative_cooling(initial_temperature, temperature_decay, iterations),
            CoolingMethod::LinearMultiplicativeCooling => linear_multiplicative_cooling(initial_temperature, temperature_decay, iterations),
            CoolingMethod::QuadraticMultiplicativeCooling => quadratic_multiplicative_cooling(initial_temperature, temperature_decay, iterations),
        };
        iterations += 1;
    }
    solution
}

fn write_result(output_path: String, output_message: &mut String) {
    let mut output_file = match OpenOptions::new().read(true).write(true).truncate(true).create(true).open(&output_path) {
        Ok(file) => file,
        Err(_) => panic!("Failed to open or create file."),
    };

    if let Err(e) = output_file.write_all(output_message.as_bytes()) {
        panic!("Failed to write to file: {}", e);
    }
}

fn main() {
    let start = Instant::now();
    let args = get_args();
    let input_path = args.input.expect("Missing argument.");
    let output_path = args.output.expect("Missing argument.");
    let config_path = args.config.expect("Missing argument.");
    let mut output_message = String::new();
    let points = read_xlsx(input_path);
    let config = read_config(config_path);
    let distance = calc_points_distance(&points);
    let solution = simulation_annealing(&points, &distance, &config, &mut output_message);
    let line: Vec<String> = solution.iter().map(|element| element.to_string()).collect();
    output_message.push_str(&format!("{}\n", line.join(" ")));
    output_message.push_str(&format!("Path length = {}\n", calc_path_length(&distance, &solution)));
    output_message.push_str(&format!("Cost time = {:?}\n", start.elapsed()));
    write_result(output_path, &mut output_message);
}
