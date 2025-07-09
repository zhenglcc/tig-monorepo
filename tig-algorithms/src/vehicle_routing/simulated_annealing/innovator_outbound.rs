/*!
Copyright 2024 Chad Blanchard

Licensed under the TIG Innovator Outbound Game License v1.0 (the "License"); you 
may not use this file except in compliance with the License. You may obtain a copy 
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

// TIG's UI uses the pattern `tig_challenges::<challenge_name>` to automatically detect your algorithm's challenge
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tig_challenges::vehicle_routing::*;

#[derive(Clone, Debug)]
struct MySubSolution {
    routes: Vec<Vec<usize>>,
    cost: f64,
}

impl MySubSolution {
    fn new(routes: Vec<Vec<usize>>, distance_matrix: &[Vec<f64>]) -> Self {
        let cost = calculate_cost(&routes, distance_matrix);
        Self { routes, cost }
    }
}


pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let mut solution = Solution {
        sub_solutions: Vec::new(),
    };
    for sub_instance in &challenge.sub_instances {
        match solve_sub_instance(sub_instance)? {
            Some(sub_solution) => solution.sub_solutions.push(sub_solution),
            None => return Ok(None),
        }
    }
    Ok(Some(solution))
}

pub fn solve_sub_instance(challenge: &SubInstance) ->  anyhow::Result<Option<SubSolution>> {
    let distance_matrix = challenge.distance_matrix.iter().map(|row| row.iter().map(|&d| d as f64).collect()).collect::<Vec<Vec<f64>>>();
    let num_nodes = challenge.difficulty.num_nodes;

    let mut rng = StdRng::seed_from_u64(42);
    let initial_solution = generate_initial_solution(num_nodes, &mut rng);
    let mut current_solution = MySubSolution::new(initial_solution.clone(), &distance_matrix);
    let mut best_solution = current_solution.clone();

    let max_iterations = 10000;
    let mut temperature = 100.0;
    let cooling_rate = 0.995;

    for _ in 0..max_iterations {
        let new_solution = generate_neighbor(&current_solution.routes, &mut rng);
        let new_solution = MySubSolution::new(new_solution, &distance_matrix);

        if acceptance_probability(current_solution.cost, new_solution.cost, temperature) > rng.gen() {
            current_solution = new_solution.clone();
        }

        if new_solution.cost < best_solution.cost {
            best_solution = new_solution;
        }

        temperature *= cooling_rate;
    }

    Ok(Some(SubSolution {
        routes: best_solution.routes,
    }))
}

fn generate_initial_solution(num_nodes: usize, rng: &mut StdRng) -> Vec<Vec<usize>> {
    let mut nodes: Vec<usize> = (1..num_nodes).collect();
    nodes.shuffle(rng);
    let mut routes = vec![vec![0]];
    for node in nodes {
        routes.last_mut().unwrap().push(node);
        if routes.last().unwrap().len() >= 5 { // Adjust this value based on your constraints
            routes.push(vec![0]);
        }
    }
    for route in &mut routes {
        route.push(0);
    }
    routes
}

fn generate_neighbor(routes: &[Vec<usize>], rng: &mut StdRng) -> Vec<Vec<usize>> {
    let mut new_routes = routes.to_vec();
    let route_idx = rng.gen_range(0..new_routes.len());
    if new_routes[route_idx].len() > 3 {
        let i = rng.gen_range(1..new_routes[route_idx].len() - 1);
        let j = rng.gen_range(1..new_routes[route_idx].len() - 1);
        new_routes[route_idx].swap(i, j);
    }
    new_routes
}

fn acceptance_probability(current_cost: f64, new_cost: f64, temperature: f64) -> f64 {
    if new_cost < current_cost {
        1.0
    } else {
        ((current_cost - new_cost) / temperature).exp()
    }
}

fn calculate_cost(routes: &[Vec<usize>], distance_matrix: &[Vec<f64>]) -> f64 {
    let mut cost = 0.0;
    for route in routes {
        for i in 0..route.len() - 1 {
            cost += distance_matrix[route[i]][route[i + 1]];
        }
    }
    cost
}