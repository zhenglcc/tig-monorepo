/*!
Copyright 2024 Lump Picasso

Licensed under the TIG Commercial License v1.0 (the "License"); you 
may not use this file except in compliance with the License. You may obtain a copy 
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

use std::collections::HashSet;
use tig_challenges::knapsack::*;


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

pub fn solve_sub_instance(challenge: &SubInstance) -> anyhow::Result<Option<SubSolution>> {
    let n = challenge.difficulty.num_items;
    let mut pairs = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            pairs.push((i, j));
        }
    }
    let weights: Vec<u32> = pairs
        .iter()
        .map(|(i, j)| challenge.weights[*i] + challenge.weights[*j])
        .collect();
    let values: Vec<u32> = pairs
        .iter()
        .map(|(i, j)| challenge.values[*i] + challenge.values[*j])
        .collect();
    let ratios: Vec<f64> = weights
        .iter()
        .zip(values.iter())
        .map(|(w, v)| *v as f64 / *w as f64)
        .collect();
    let mut sorted_value_to_weight_ratio: Vec<usize> = (0..n).collect();
    sorted_value_to_weight_ratio.sort_by(|&a, &b| ratios[a].partial_cmp(&ratios[b]).unwrap());

    let items = HashSet::<usize>::new();
    let mut total_weight = 0;
    let max_weight = challenge.max_weight;
    for &idx in &sorted_value_to_weight_ratio {
        let mut additional_weight = 0;
        let p = pairs[idx];
        if !items.contains(&p.0) {
            additional_weight += challenge.weights[p.0];
        }
        if !items.contains(&p.1) {
            additional_weight += challenge.weights[p.1];
        }
        if total_weight + additional_weight > max_weight {
            continue;
        }
        total_weight += additional_weight;
    }
    Ok(Some(SubSolution {
        items: items.into_iter().collect(),
    }))
}