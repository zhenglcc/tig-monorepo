/*!
Copyright 2024 JS

Licensed under the TIG Commercial License v1.0 (the "License"); you 
may not use this file except in compliance with the License. You may obtain a copy 
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

// TIG's UI uses the pattern `tig_challenges::<challenge_name>` to automatically detect your algorithm's challenge
use anyhow::{anyhow, Result};
use tig_challenges::knapsack::*;
use std::cmp::Ordering;


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

pub fn solve_sub_instance(challenge: &SubInstance) -> Result<Option<SubSolution>> {
    let num_items = challenge.weights.len();
    let mut remaining_capacity = challenge.max_weight as i32; 
    let mut densities = vec![0.0; num_items];
    let mut selected_items = Vec::with_capacity(num_items/2);

    for i in 0..num_items {
        densities[i] = challenge.values[i] as f64 / challenge.weights[i] as f64;
    }

    let mut sorted_indices: Vec<usize> = (0..num_items).collect();
    sorted_indices.sort_unstable_by(|&i, &j| densities[i].partial_cmp(&densities[j]).unwrap_or(Ordering::Equal));

    while let Some(i) = sorted_indices.pop() {
        let weight = challenge.weights[i] as i32;
        if weight > remaining_capacity {
            continue;
        }

        selected_items.push(i);
        remaining_capacity -= weight;

        for &j in &sorted_indices {
            let joint_profit = challenge.interaction_values[i][j] as f64;
            densities[j] += joint_profit / challenge.weights[j] as f64;
        }

        sorted_indices.sort_unstable_by(|&i, &j| densities[i].partial_cmp(&densities[j]).unwrap_or(Ordering::Equal));
    }

    Ok(Some(SubSolution { items: selected_items }))
}




// Important! Do not include any tests in this file, it will result in your submission being rejected