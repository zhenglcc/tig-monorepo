/*!Copyright 2025 Rootz

Licensed under the TIG Benchmarker Outbound Game License v2.0 (the "License"); you
may not use this file except in compliance with the License. You may obtain a copy
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/
use cudarc::{
    driver::{safe::LaunchConfig, CudaModule, CudaStream, PushKernelArg},
    runtime::sys::cudaDeviceProp,
};
use std::sync::Arc;
use tig_challenges::hypergraph::*;

pub fn solve_challenge(
    challenge: &Challenge,
    module: Arc<CudaModule>,
    stream: Arc<CudaStream>,
    prop: &cudaDeviceProp,
) -> anyhow::Result<Option<Solution>> {
    let mut solution = Solution {
        sub_solutions: Vec::new(),
    };
    for sub_instance in &challenge.sub_instances {
        match solve_sub_instance(sub_instance, module.clone(), stream.clone(), prop)? {
            Some(sub_solution) => solution.sub_solutions.push(sub_solution),
            None => return Ok(None),
        }
    }
    Ok(Some(solution))
}

pub fn solve_sub_instance(
    instance: &SubInstance,
    module: Arc<CudaModule>,
    stream: Arc<CudaStream>,
    prop: &cudaDeviceProp,
) -> anyhow::Result<Option<SubSolution>> {
    let block_size = std::cmp::min(256, prop.maxThreadsPerBlock as u32);
    
    let hyperedge_cluster_kernel = module.load_function("hyperedge_clustering")?;
    let compute_preferences_kernel = module.load_function("compute_node_preferences")?;
    let execute_assignments_kernel = module.load_function("execute_node_assignments")?;
    let compute_moves_kernel = module.load_function("compute_refinement_moves")?;
    let execute_moves_kernel = module.load_function("execute_refinement_moves")?;
    let balance_kernel = module.load_function("balance_final")?;
    
    let cfg = LaunchConfig {
        grid_dim: ((instance.num_nodes as u32 + block_size - 1) / block_size, 1, 1),
        block_dim: (block_size, 1, 1),
        shared_mem_bytes: 0,
    };
    
    let one_thread_cfg = LaunchConfig {
        grid_dim: (1, 1, 1),
        block_dim: (1, 1, 1),
        shared_mem_bytes: 0,
    };
    
    let num_hedge_clusters = if instance.difficulty.num_hyperedges < 50000 {
        let thousands = instance.difficulty.num_hyperedges / 1000;
        match thousands {
            5 => 2, 6 => 2, 7 => 2, 8 => 2, 9 => 2, 10 => 2, 11 => 4, 12 => 6, 13 => 2, 14 => 2,
            15 => 6, 16 => 2, 17 => 2, 18 => 2, 19 => 4, 20 => 2, 21 => 4, 22 => 4, 23 => 4, 24 => 2,
            25 => 6, 26 => 2, 27 => 2, 28 => 2, 29 => 2, 30 => 2, 31 => 2, 32 => 2, 33 => 8, 34 => 8,
            35 => 8, 36 => 4, 37 => 8, 38 => 4, 39 => 6, 40 => 2, 41 => 2, 42 => 2, 43 => 6, 44 => 2,
            45 => 2, 46 => 2, 47 => 2, 48 => 2, 49 => 2,
            _ => if thousands < 5 { 2 } else { 8 }
        }
    } else {
        std::env::var("CLUSTER_SIZE").unwrap_or_else(|_| "8".to_string()).parse::<usize>().unwrap_or(8)
    };
    
    let mut d_hyperedge_clusters = stream.alloc_zeros::<i32>(instance.difficulty.num_hyperedges as usize)?;
    let mut d_partition = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_nodes_in_part = stream.alloc_zeros::<i32>(instance.num_parts as usize)?;
    
    let mut d_pref_nodes = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_pref_parts = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_pref_gains = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_pref_priorities = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    
    let mut d_move_nodes = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_move_parts = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_move_gains = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_move_priorities = stream.alloc_zeros::<i32>(instance.num_nodes as usize)?;
    let mut d_num_valid_moves = stream.alloc_zeros::<i32>(1)?;
    
    let num_threads = ((instance.num_nodes as u32 + block_size - 1) / block_size) * block_size;
    let buffer_size = (num_threads * 3000) as usize;
    let mut d_global_edge_flags_low = stream.alloc_zeros::<u64>(buffer_size)?;
    
    let mut d_global_edge_flags_high = if instance.num_parts > 64 {
        stream.alloc_zeros::<u64>(buffer_size)?
    } else {
        stream.alloc_zeros::<u64>(1)?
    };
    
    unsafe {
        stream.launch_builder(&hyperedge_cluster_kernel)
            .arg(&(instance.difficulty.num_hyperedges as i32))
            .arg(&(num_hedge_clusters as i32))
            .arg(&instance.d_hyperedge_nodes)
            .arg(&instance.d_hyperedge_offsets)
            .arg(&mut d_hyperedge_clusters)
            .launch(LaunchConfig {
                grid_dim: ((instance.difficulty.num_hyperedges as u32 + block_size - 1) / block_size, 1, 1),
                block_dim: (block_size, 1, 1),
                shared_mem_bytes: 0,
            })?;
    }
    stream.synchronize()?;
    
    unsafe {
        stream.launch_builder(&compute_preferences_kernel)
            .arg(&(instance.num_nodes as i32))
            .arg(&(instance.num_parts as i32))
            .arg(&(num_hedge_clusters as i32))
            .arg(&instance.d_node_hyperedges)
            .arg(&instance.d_node_offsets)
            .arg(&d_hyperedge_clusters)
            .arg(&instance.d_hyperedge_offsets)
            .arg(&mut d_pref_nodes)
            .arg(&mut d_pref_parts)
            .arg(&mut d_pref_gains)
            .arg(&mut d_pref_priorities)
            .launch(cfg.clone())?;
    }
    stream.synchronize()?;
    
    let pref_nodes = stream.memcpy_dtov(&d_pref_nodes)?;
    let pref_parts = stream.memcpy_dtov(&d_pref_parts)?;
    let pref_priorities = stream.memcpy_dtov(&d_pref_priorities)?;
    
    let mut indices: Vec<usize> = (0..instance.num_nodes as usize).collect();
    indices.sort_by(|&a, &b| pref_priorities[b].cmp(&pref_priorities[a]));
    
    let sorted_nodes: Vec<i32> = indices.iter().map(|&i| pref_nodes[i]).collect();
    let sorted_parts: Vec<i32> = indices.iter().map(|&i| pref_parts[i]).collect();
    
    let d_sorted_nodes = stream.memcpy_stod(&sorted_nodes)?;
    let d_sorted_parts = stream.memcpy_stod(&sorted_parts)?;
    
    unsafe {
        stream.launch_builder(&execute_assignments_kernel)
            .arg(&(instance.num_nodes as i32))
            .arg(&(instance.num_parts as i32))
            .arg(&(instance.max_part_size as i32))
            .arg(&d_sorted_nodes)
            .arg(&d_sorted_parts)
            .arg(&mut d_partition)
            .arg(&mut d_nodes_in_part)
            .launch(one_thread_cfg.clone())?;
    }
    stream.synchronize()?;
    
    let mut valid_moves: Vec<(i32, i32, i32)> = Vec::with_capacity(instance.num_nodes as usize);
    let mut sorted_move_nodes: Vec<i32> = Vec::with_capacity(instance.num_nodes as usize);
    let mut sorted_move_parts: Vec<i32> = Vec::with_capacity(instance.num_nodes as usize);
    
    for round in 0..100 {
        unsafe {
            stream.launch_builder(&compute_moves_kernel)
                .arg(&(instance.num_nodes as i32))
                .arg(&(instance.num_parts as i32))
                .arg(&(instance.max_part_size as i32))
                .arg(&(instance.difficulty.num_hyperedges as i32))
                .arg(&instance.d_node_hyperedges)
                .arg(&instance.d_node_offsets)
                .arg(&instance.d_hyperedge_nodes)
                .arg(&instance.d_hyperedge_offsets)
                .arg(&d_partition)
                .arg(&d_nodes_in_part)
                .arg(&mut d_move_nodes)
                .arg(&mut d_move_parts)
                .arg(&mut d_move_gains)
                .arg(&mut d_move_priorities)
                .arg(&mut d_num_valid_moves)
                .arg(&round)
                .arg(&mut d_global_edge_flags_low)
                .arg(&mut d_global_edge_flags_high)
                .launch(cfg.clone())?;
        }
        stream.synchronize()?;
        
        let num_valid_moves = stream.memcpy_dtov(&d_num_valid_moves)?[0];
        if num_valid_moves == 0 {
            break;
        }
        
        let move_gains = stream.memcpy_dtov(&d_move_gains)?;
        let valid_indices: Vec<usize> = move_gains.iter().enumerate()
            .filter(|(_, &gain)| gain > 0)
            .map(|(i, _)| i)
            .collect();
        
        if valid_indices.is_empty() {
            break;
        }
        
        let move_nodes = stream.memcpy_dtov(&d_move_nodes)?;
        let move_parts = stream.memcpy_dtov(&d_move_parts)?;
        let move_priorities = stream.memcpy_dtov(&d_move_priorities)?;
        
        valid_moves.clear();
        for &i in &valid_indices {
            valid_moves.push((move_nodes[i], move_parts[i], move_priorities[i]));
        }
        
        valid_moves.sort_by(|a, b| b.2.cmp(&a.2));
        
        sorted_move_nodes.clear();
        sorted_move_parts.clear();
        sorted_move_nodes.extend(valid_moves.iter().map(|&(node, _, _)| node));
        sorted_move_parts.extend(valid_moves.iter().map(|&(_, part, _)| part));
        
        let d_sorted_move_nodes = stream.memcpy_stod(&sorted_move_nodes)?;
        let d_sorted_move_parts = stream.memcpy_stod(&sorted_move_parts)?;
        let mut d_moves_executed = stream.alloc_zeros::<i32>(1)?;
        
        unsafe {
            stream.launch_builder(&execute_moves_kernel)
                .arg(&(sorted_move_nodes.len() as i32))
                .arg(&d_sorted_move_nodes)
                .arg(&d_sorted_move_parts)
                .arg(&(instance.max_part_size as i32))
                .arg(&mut d_partition)
                .arg(&mut d_nodes_in_part)
                .arg(&mut d_moves_executed)
                .launch(one_thread_cfg.clone())?;
        }
        stream.synchronize()?;
        
        let moves_executed = stream.memcpy_dtov(&d_moves_executed)?[0];
        if moves_executed == 0 {
            break;
        }
    }
    
    unsafe {
        stream.launch_builder(&balance_kernel)
            .arg(&(instance.num_nodes as i32))
            .arg(&(instance.num_parts as i32))
            .arg(&1)
            .arg(&(instance.max_part_size as i32))
            .arg(&mut d_partition)
            .arg(&mut d_nodes_in_part)
            .launch(one_thread_cfg.clone())?;
    }
    stream.synchronize()?;
    
    let partition = stream.memcpy_dtov(&d_partition)?;
    let partition_u32: Vec<u32> = partition.iter().map(|&x| x as u32).collect();
    
    Ok(Some(SubSolution { partition: partition_u32 }))
}
