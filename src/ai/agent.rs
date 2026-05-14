use crate::ai::ga::GeneticAlgorithm;
use crate::ai::network::{B, DinoNet};
use crate::ai::simulation::Simulation;
use crate::constants;
use macroquad::prelude::*;

pub struct AgentState {
    pub sim: Simulation,
    pub network: DinoNet<B>,
}

pub fn create_agents(ga: &GeneticAlgorithm) -> Vec<AgentState> {
    ga.population
        .iter()
        .map(|indiv| AgentState {
            sim: Simulation::new(),
            network: DinoNet::from_weights(&indiv.genes),
        })
        .collect()
}

pub struct WorldAgent {
    pub col: usize,
    pub row: usize,
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub on_ground: bool,
    pub alive: bool,
    pub network: DinoNet<B>,
    pub score: u32,
    pub score_timer: f32,
    pub fitness: f32,
    pub ground_y: f32,
}

pub fn create_world_agents(ga: &GeneticAlgorithm) -> Vec<WorldAgent> {
    let mut agents = Vec::with_capacity(ga.population.len());
    // All agents use the EXACT same ground and hitbox as the real Player
    let base_x = screen_width() * 0.10;
    let real_ground = constants::ground_y() - constants::PLAYER_SIZE.1;
    for (i, indiv) in ga.population.iter().enumerate() {
        let col = i % 10;
        // Visual depth offset (±6px for visual layering only, physics unaffected)
        let row = i / 10;
        let vis_offset = (row as f32 - 4.5) * 1.5;
        agents.push(WorldAgent {
            col,
            row,
            x: base_x,
            y: real_ground + vis_offset,
            vy: 0.0,
            on_ground: true,
            alive: true,
            network: DinoNet::from_weights(&indiv.genes),
            score: 0,
            score_timer: 0.0,
            fitness: 0.0,
            ground_y: real_ground,
        });
    }
    agents
}
