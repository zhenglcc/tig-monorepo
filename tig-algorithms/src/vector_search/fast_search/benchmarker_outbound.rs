/*!
Copyright 2024 Chibs

Licensed under the TIG Benchmarker Outbound Game License v1.0 (the "License"); you 
may not use this file except in compliance with the License. You may obtain a copy 
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

use anyhow::{anyhow, Result};
use cudarc::{
    driver::{safe::LaunchConfig, CudaModule, CudaStream, PushKernelArg},
    runtime::sys::cudaDeviceProp,
};
use std::sync::Arc;
use tig_challenges::vector_search::{Challenge, Solution};

pub fn solve_challenge(
    challenge: &Challenge,
    module: Arc<CudaModule>,
    stream: Arc<CudaStream>,
    prop: &cudaDeviceProp,
) -> Result<Option<Solution>> {
    Err(anyhow!("This algorithm is no longer compatible."))
}

// Old code that is no longer compatible
#[cfg(none)]
mod dead_code {
   // TIG's UI uses the pattern `tig_challenges::<challenge_name>` to automatically detect your algorithm's challenge
   use anyhow::Result;
   use tig_challenges::vector_search::{Challenge, Solution};

   fn squared_distance(v1: &[f32], v2: &[f32]) -> f32 {
       v1.iter()
           .zip(v2.iter())
           .map(|(a, b)| (a - b) * (a - b))
           .sum()
   }

   pub fn solve_challenge(challenge: &Challenge) -> Result<Option<Solution>> {
       let max_distance_sq = challenge.max_distance * challenge.max_distance;

       let indexes: Vec<usize> = challenge
           .query_vectors
           .iter()
           .filter_map(|query| {
               challenge
                   .vector_database
                   .iter()
                   .enumerate()
                   .find_map(|(i, vector)| {
                       if squared_distance(query, vector) <= max_distance_sq {
                           Some(i)
                       } else {
                           None
                       }
                   })
           })
           .collect();

       if indexes.len() == challenge.query_vectors.len() {
           Ok(Some(Solution { indexes }))
       } else {
           Ok(None)
       }
   }

   // Important! Do not include any tests in this file, it will result in your submission being rejected
}