use super::tree::Tree;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Data {
    pub t: f64,
    pub c: usize,
    pub positions: Vec<[f64; 3]>,
    pub speeds: Vec<[f64; 3]>,
    pub energy: Vec<[f64; 2]>, // particle's energy [kinetic, potential]
    pub rayons: [f64; 3],
    pub inertia_matrix: [f64; 9],
    pub total_energy: f64, // system energy
    pub virial: f64,
    pub dynamical_time: f64,
    pub espilon: f64,
}

impl Data {
    pub fn new(t: f64, c: usize, tree: &Tree) -> Data {
        //! Create a new Data instance.
        //! t is the current time, c the current iteration number,
        //! and all other relevant data is *copied* from tree.

        let mut positions = Vec::new();
        let mut speeds = Vec::new();

        for i in 0..tree.nb_save {
            positions.push([
                tree.particules[i].position[0] - tree.center[0],
                tree.particules[i].position[1] - tree.center[1],
                tree.particules[i].position[2] - tree.center[2],
            ]);

            speeds.push([
                tree.particules[i].speed[0],
                tree.particules[i].speed[1],
                tree.particules[i].speed[2],
            ]);

            energy.push([
                tree.particules[i].cinetic,
                tree.particules[i].mass * tree.particules[i].potential,
            ]);
        }
        Data {
            t,
            c,
            positions,
            speeds,
            energy,
            rayons: tree.rayons,
            inertia_matrix: tree.inertia_matrix,
            total_energy: tree.energy,
            virial: tree.virial,
            dynamical_time: tree.dynamical_time,
            espilon: tree.epsilon,
        }
    }

    pub fn new_empty() -> Data {
        //! Create a dumb empty new Data instance
        //! with 0 or empty values.
        Data {
            t: 0f64,
            c: 0usize,
            positions: Vec::new(),
            speeds: Vec::new(),
            energy: Vec::new(),
            rayons: [0f64; 3],
            inertia_matrix: [0f64; 9],
            total_energy: 0f64,
            virial: 0f64,
            dynamical_time: 0f64,
            espilon: 0f64,
        }
    }
}
