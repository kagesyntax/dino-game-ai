use crate::constants;
use crate::entity::Enemies;
use burn::module::Module;
use burn::nn::{self, LinearConfig};
use burn::prelude::*;
use burn::tensor::Shape;
use burn::tensor::activation;
use macroquad::prelude::*;

pub const INPUT: usize = 7;
pub const HIDDEN: usize = 12;
pub const NUM_PARAMS: usize = INPUT * HIDDEN + HIDDEN + HIDDEN + 1;

pub type B = burn::backend::NdArray<f32>;

#[derive(Module, Debug)]
pub struct DinoNet<B: Backend> {
    pub fc1: nn::Linear<B>,
    pub fc2: nn::Linear<B>,
}

impl<B: Backend> DinoNet<B> {
    pub fn new(device: &B::Device) -> Self {
        Self {
            fc1: LinearConfig::new(INPUT, HIDDEN).init(device),
            fc2: LinearConfig::new(HIDDEN, 1).init(device),
        }
    }

    pub fn forward(&self, input: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.fc1.forward(input);
        let x = activation::relu(x);
        self.fc2.forward(x)
    }
}

impl DinoNet<B> {
    pub fn save_weights(&self, path: &str) -> std::io::Result<()> {
        let weights = self.get_weights();
        let json = serde_json::to_string(&weights).unwrap();
        std::fs::write(path, json)
    }

    pub fn load_weights(path: &str) -> Option<Vec<f32>> {
        if let Ok(json) = std::fs::read_to_string(path) {
            serde_json::from_str(&json).ok()
        } else {
            None
        }
    }

    pub fn from_weights(weights: &[f32]) -> Self {
        let device = &Default::default();
        let mut record = DinoNet::new(device).into_record();

        let (w1, r) = weights.split_at(INPUT * HIDDEN);
        let (b1, r) = r.split_at(HIDDEN);
        let (w2, b2) = r.split_at(HIDDEN);

        // In Burn 0.16, Linear weight is [d_in, d_out] and forward is xW + b
        record.fc1.weight = record.fc1.weight.map(|_| {
            Tensor::from_data(
                TensorData::new(w1.to_vec(), Shape::new([INPUT, HIDDEN])),
                device,
            )
        });
        record.fc1.bias = record.fc1.bias.map(|param| {
            param.map(|_| {
                Tensor::from_data(TensorData::new(b1.to_vec(), Shape::new([HIDDEN])), device)
            })
        });
        record.fc2.weight = record.fc2.weight.map(|_| {
            Tensor::from_data(
                TensorData::new(w2.to_vec(), Shape::new([HIDDEN, 1])),
                device,
            )
        });
        record.fc2.bias = record.fc2.bias.map(|param| {
            param.map(|_| Tensor::from_data(TensorData::new(b2.to_vec(), Shape::new([1])), device))
        });

        DinoNet::new(device).load_record(record)
    }

    pub fn get_weights(&self) -> Vec<f32> {
        let mut weights = Vec::with_capacity(NUM_PARAMS);

        let w1 = self.fc1.weight.val().into_data().to_vec::<f32>().unwrap();
        let b1 = self
            .fc1
            .bias
            .as_ref()
            .unwrap()
            .val()
            .into_data()
            .to_vec::<f32>()
            .unwrap();
        let w2 = self.fc2.weight.val().into_data().to_vec::<f32>().unwrap();
        let b2 = self
            .fc2
            .bias
            .as_ref()
            .unwrap()
            .val()
            .into_data()
            .to_vec::<f32>()
            .unwrap();

        weights.extend(w1);
        weights.extend(b1);
        weights.extend(w2);
        weights.extend(b2);

        weights
    }

    pub fn predict(&self, features: &[f32; INPUT]) -> bool {
        let device = &Default::default();
        let data = TensorData::new(features.to_vec(), Shape::new([1, INPUT]));
        let input = Tensor::<B, 2>::from_data(data, device);
        let output = self.forward(input);
        let vals = output
            .into_data()
            .to_vec::<f32>()
            .expect("Failed to get tensor data");
        vals[0] > 0.0
    }
}

pub fn ai_features_at(
    agent_x: f32,
    ai_y: f32,
    ai_vy: f32,
    on_ground: bool,
    score: u32,
    enemies: &Enemies,
) -> [f32; INPUT] {
    let mut min_dist = f32::MAX;
    let mut obs_w = 0.0;
    let mut obs_h = 0.0;
    for hr in enemies.hitboxes() {
        let dist = hr.x - (agent_x + constants::PLAYER_SIZE.0);
        if dist >= -constants::PLAYER_SIZE.0 && dist < min_dist {
            min_dist = dist;
            obs_w = hr.w;
            obs_h = hr.h;
        }
    }
    let sw = screen_width();
    let sh = screen_height();
    let gs_y = constants::ground_y();
    let gs = (7.0 + score as f32 / 100.0).min(17.0);
    [
        (min_dist / sw).clamp(0.0, 1.0),
        (obs_w / sw).clamp(0.0, 1.0),
        (obs_h / sh).clamp(0.0, 1.0),
        ((gs_y - ai_y) / 200.0).clamp(0.0, 1.0),
        (ai_vy / 800.0).clamp(-1.0, 1.0),
        if on_ground { 1.0 } else { 0.0 },
        ((gs - 7.0) / 10.0).clamp(0.0, 1.0),
    ]
}
