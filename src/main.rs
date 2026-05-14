use macroquad::prelude::*;

mod ai;
mod constants;
mod entity;
use ai::{
    B, DinoNet, GeneticAlgorithm, HIDDEN, INPUT, WorldAgent, ai_features_at, create_agents,
    create_world_agents,
};
use entity::{Enemies, Player};

#[derive(Clone)]
enum Screen {
    Menu,
    HumanPlay,
    AiTrain,
    HumanVsAi,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Dino Game".to_owned(),
        window_width: 800,
        window_height: 400,
        ..Default::default()
    }
}

const WEIGHTS_FILE: &str = "best_brain.json";

#[macroquad::main(window_conf)]
async fn main() {
    let sprite = load_texture("assets/sprite.png")
        .await
        .expect("Failed to load sprite.png — make sure it exists in the assets/ folder");
    sprite.set_filter(FilterMode::Nearest);

    let mut screen = Screen::Menu;

    // Human play state
    let mut player = Player::new();
    let mut enemies = Enemies::new();
    let mut human_score: u32 = 0;
    let mut high_score: u32 = 0;
    let mut ground_offset: f32 = 0.0;
    let mut anim_timer: f32 = 0.0;
    let mut anim_frame: bool = false;
    let mut score_timer: f32 = 0.0;
    let mut show_hitboxes: bool = false;

    // AI training state
    let mut ga: Option<GeneticAlgorithm> = None;
    let mut world_agents: Vec<WorldAgent> = Vec::new();
    let mut world_enemies: Option<Enemies> = None;
    let mut deaths_this_gen: usize = 0;
    let mut show_gen_ended: f32 = 0.0;
    let mut world_best_score: u32 = 0;
    let mut saved_ai_brain: Option<Vec<f32>> = ai::DinoNet::<ai::B>::load_weights(WEIGHTS_FILE);

    // Human vs AI state
    let mut vs_human_player = Player::new();
    let mut vs_ai_player = Player::new();
    let mut vs_enemies = Enemies::new();
    let mut vs_human_score: u32 = 0;
    let mut vs_ai_score: u32 = 0;
    let mut vs_human_alive: bool = true;
    let mut vs_ai_alive: bool = true;
    let mut vs_ai_network: Option<ai::DinoNet<burn::backend::NdArray<f32>>> = None;
    let mut vs_score_timer: f32 = 0.0;
    let mut vs_ground_offset: f32 = 0.0;
    let mut vs_winner: u8 = 0; // 0=playing, 1=human, 2=AI

    loop {
        let dt = get_frame_time();
        clear_background(WHITE);

        if is_key_pressed(KeyCode::F1) {
            show_hitboxes = !show_hitboxes;
        }

        match screen.clone() {
            Screen::Menu => {
                let title = "DINO GAME";
                let tw = measure_text(title, None, 60u16, 1.).width;
                draw_text(title, screen_width() / 2. - tw / 2., 120., 60., BLACK);

                let items = ["1. Human Play", "2. AI Training", "3. Human vs AI"];
                for (i, item) in items.iter().enumerate() {
                    let iw = measure_text(item, None, 30u16, 1.).width;
                    draw_text(
                        item,
                        screen_width() / 2. - iw / 2.,
                        220. + i as f32 * 50.,
                        30.,
                        DARKGRAY,
                    );
                }

                if is_key_pressed(KeyCode::Key1) {
                    player = Player::new();
                    enemies = Enemies::new();
                    human_score = 0;
                    score_timer = 0.0;
                    ground_offset = 0.0;
                    screen = Screen::HumanPlay;
                }
                if is_key_pressed(KeyCode::Key2) {
                    ga = Some(GeneticAlgorithm::new());
                    world_agents.clear();
                    world_best_score = 0;
                    deaths_this_gen = 0;
                    world_enemies = Some(Enemies::new());
                    if let Some(ref g) = ga {
                        world_agents = create_world_agents(g);
                    }
                    screen = Screen::AiTrain;
                }
                if is_key_pressed(KeyCode::Key3) {
                    // Use saved brain from training, or fallback to a quick 5-gen train
                    let brain = if let Some(ref w) = saved_ai_brain {
                        Some(DinoNet::from_weights(w))
                    } else {
                        let mut dummy_ga = GeneticAlgorithm::new();
                        for _ in 0..5 {
                            let mut pop = create_agents(&dummy_ga);
                            for ag in pop.iter_mut() {
                                for _ in 0..1200 {
                                    let gs = (7.0_f32 + ag.sim.score() as f32 / 100.0).min(17.0);
                                    let feats = ag.sim.features();
                                    let jump = ag.network.predict(&feats);
                                    ag.sim.step(1.0 / 60.0, jump, gs * 60.0);
                                    if ag.sim.check_collision() {
                                        break;
                                    }
                                }
                            }
                            for ag in &pop {
                                dummy_ga.record_fitness(ag.sim.score() as f32 + ag.sim.elapsed());
                            }
                        }
                        Some(dummy_ga.best_network())
                    };
                    vs_ai_network = brain;
                    vs_human_player = Player::new();
                    vs_ai_player = Player::new();
                    vs_enemies = Enemies::new();
                    vs_human_score = 0;
                    vs_ai_score = 0;
                    vs_human_alive = true;
                    vs_ai_alive = true;
                    vs_score_timer = 0.0;
                    vs_ground_offset = 0.0;
                    vs_winner = 0;
                    screen = Screen::HumanVsAi;
                }
            }
            Screen::HumanPlay => {
                let gs = (7.0 + human_score as f32 / 100.0).min(17.0);
                ground_offset = (ground_offset + gs * 60.0 * dt) % constants::GROUND_W;
                enemies.update(dt, gs * 60.0);
                player.update(dt);
                anim_timer += dt;
                if anim_timer >= 0.1 {
                    anim_frame = !anim_frame;
                    anim_timer -= 0.1;
                }
                score_timer += dt;
                if score_timer >= 0.1 {
                    human_score += 1;
                    score_timer -= 0.1;
                }
                let pr = Rect::new(
                    player.rect().x + 10.,
                    player.rect().y + 10.,
                    player.rect().w - 20.,
                    player.rect().h - 20.,
                );
                if enemies.check_collision(&pr) {
                    if human_score > high_score {
                        high_score = human_score;
                    }
                    screen = Screen::Menu;
                }

                draw_ground(&sprite, ground_offset);
                enemies.draw(&sprite);
                player.draw(&sprite, anim_frame);

                let st = format!("SCORE: {}", human_score);
                draw_text(&st, 15., 30., 20., BLACK);
                let hs = format!("HI: {}", high_score);
                let hw = measure_text(&hs, None, 20u16, 1.).width;
                draw_text(&hs, screen_width() - hw - 15., 30., 20., BLACK);

                let back = "[ESC] Menu";
                draw_text(back, 15., screen_height() - 15., 15., DARKGRAY);

                if is_key_pressed(KeyCode::Escape) {
                    screen = Screen::Menu;
                }

                if show_hitboxes {
                    let pr = player.rect();
                    draw_rectangle_lines(pr.x, pr.y, pr.w, pr.h, 2., GREEN);
                    for hr in enemies.hitboxes() {
                        draw_rectangle_lines(hr.x, hr.y, hr.w, hr.h, 2., RED);
                    }
                    draw_text("HITBOXES ON (F1)", 15., 55., 15., DARKGRAY);
                }
            }
            Screen::AiTrain => {
                if let (Some(ref mut g), Some(ref mut enemies)) =
                    (ga.as_mut(), world_enemies.as_mut())
                {
                    let max_score = world_agents
                        .iter()
                        .map(|a| a.score)
                        .max()
                        .unwrap_or(1)
                        .max(1);
                    let gs = (7.0 + max_score as f32 / 100.0).min(17.0);
                    let gs_px = gs * 60.0;
                    enemies.update(dt, gs_px);

                    // Update all alive agents
                    for agent in world_agents.iter_mut().filter(|a| a.alive) {
                        let feats = ai_features_at(
                            agent.x,
                            agent.y,
                            agent.vy,
                            agent.on_ground,
                            agent.score,
                            enemies,
                        );
                        let jump = agent.network.predict(&feats);

                        agent.vy += constants::GRAVITY * dt;
                        agent.y += agent.vy * dt;

                        if agent.y >= agent.ground_y {
                            agent.y = agent.ground_y;
                            agent.vy = 0.0;
                            agent.on_ground = true;
                        }
                        if jump && agent.on_ground {
                            agent.vy = -constants::JUMP_VEL;
                            agent.y += agent.vy * dt;
                            agent.on_ground = false;
                        }

                        // Exact same hitbox as the real Player
                        let ar = Rect::new(
                            agent.x + 5.,
                            agent.y + 5.,
                            constants::PLAYER_SIZE.0 - 10.,
                            constants::PLAYER_SIZE.1 - 10.,
                        );
                        if enemies.check_collision(&ar) || agent.score >= 5000 {
                            agent.alive = false;
                            agent.fitness = agent.score as f32;
                            let idx = agent.row * 10 + agent.col;
                            if idx < g.population.len() {
                                g.population[idx].fitness = agent.fitness;
                            }
                            deaths_this_gen += 1;
                        }

                        agent.score_timer += dt;
                        if agent.score_timer >= 0.1 {
                            agent.score += 1;
                            agent.score_timer -= 0.1;
                        }
                    }

                    if deaths_this_gen == g.population.len() {
                        g.evolve();
                        // Save best brain for Human vs AI
                        let best_idx = g
                            .population
                            .iter()
                            .enumerate()
                            .max_by(|a, b| a.1.fitness.partial_cmp(&b.1.fitness).unwrap())
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        let best_genes = g.population[best_idx].genes.clone();
                        saved_ai_brain = Some(best_genes.clone());
                        let _ = ai::DinoNet::<ai::B>::from_weights(&best_genes)
                            .save_weights(WEIGHTS_FILE);

                        world_agents = create_world_agents(g);
                        **enemies = Enemies::new();
                        deaths_this_gen = 0;
                        show_gen_ended = 1.5;
                        world_best_score = 0;
                    }

                    let alive = world_agents.iter().filter(|a| a.alive).count();
                    let curr_best_score = world_agents
                        .iter()
                        .filter(|a| a.alive)
                        .map(|a| a.score)
                        .max()
                        .unwrap_or(0);
                    if curr_best_score > world_best_score {
                        world_best_score = curr_best_score;
                    }

                    anim_timer += dt;
                    if anim_timer >= 0.1 {
                        anim_frame = !anim_frame;
                        anim_timer -= 0.1;
                    }

                    // ── Rendering (Full Screen View) ──
                    draw_ground(&sprite, 0.0);
                    enemies.draw(&sprite);

                    // Find the best agent for brain visualization
                    let best_agent_ptr = world_agents
                        .iter()
                        .filter(|a| a.alive)
                        .max_by_key(|a| a.score)
                        .map(|a| a as *const WorldAgent);

                    // Render all agents
                    for agent in world_agents.iter() {
                        if !agent.alive {
                            continue;
                        }

                        let frame_x = if !agent.on_ground {
                            constants::PLAYER_JUMP_X
                        } else if anim_frame {
                            constants::PLAYER_FRAME1_X
                        } else {
                            constants::PLAYER_FRAME2_X
                        };
                        let dp = DrawTextureParams {
                            source: Some(Rect::new(
                                frame_x,
                                0.,
                                constants::PLAYER_SRC_W,
                                constants::PLAYER_SRC_H,
                            )),
                            dest_size: Some(Vec2::new(
                                constants::PLAYER_SIZE.0,
                                constants::PLAYER_SIZE.1,
                            )),
                            ..Default::default()
                        };

                        let is_best = best_agent_ptr.is_some_and(|ptr| std::ptr::eq(ptr, agent));
                        let alpha = if is_best { 1.0 } else { 0.2 };
                        draw_texture_ex(
                            &sprite,
                            agent.x,
                            agent.y,
                            Color::new(1., 1., 1., alpha),
                            dp,
                        );

                        if is_best {
                            draw_text("BEST", agent.x, agent.y - 10.0, 12., DARKGRAY);
                        }
                    }

                    // ── Brain Visualization (CodeBullet style) ──
                    if let Some(best_idx) = world_agents
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| a.alive)
                        .max_by_key(|(_, a)| a.score)
                        .map(|(i, _)| i)
                    {
                        draw_nn(&world_agents[best_idx].network);
                    }

                    // ── HUD ──
                    draw_text(
                        &format!("Generation: {}", g.generation),
                        20.,
                        30.,
                        20.,
                        DARKGRAY,
                    );
                    draw_text(
                        &format!("Alive: {}/{}", alive, world_agents.len()),
                        20.,
                        55.,
                        20.,
                        DARKGRAY,
                    );
                    draw_text(
                        &format!("Best fitness: {:.0}", g.best_fitness),
                        20.,
                        80.,
                        20.,
                        DARKGRAY,
                    );
                    draw_text(
                        &format!("Top score: {}", world_best_score),
                        20.,
                        105.,
                        20.,
                        DARKGRAY,
                    );

                    draw_text("[ESC] Menu", 20., screen_height() - 15., 15., DARKGRAY);

                    if show_gen_ended > 0.0 {
                        let msg = format!("Generation {} evolved!", g.generation - 1);
                        let mw = measure_text(&msg, None, 30u16, 1.).width;
                        draw_text(
                            &msg,
                            screen_width() / 2. - mw / 2.,
                            150.,
                            30.,
                            Color::new(0.2, 0.2, 0.2, show_gen_ended.min(1.0)),
                        );
                        show_gen_ended -= dt;
                    }

                    if show_hitboxes {
                        for hr in enemies.hitboxes() {
                            draw_rectangle_lines(hr.x, hr.y, hr.w, hr.h, 2., RED);
                        }
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    if let Some(ref g) = ga {
                        // Pick best by SCORE — alive agents with huge scores beat dead ones
                        let best_idx = world_agents
                            .iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| {
                                let sa = if a.alive { a.score } else { a.fitness as u32 };
                                let sb = if b.alive { b.score } else { b.fitness as u32 };
                                sa.cmp(&sb)
                            })
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        let bg = g.population[best_idx].genes.clone();
                        saved_ai_brain = Some(bg.clone());
                        let _ = ai::DinoNet::<ai::B>::from_weights(&bg).save_weights(WEIGHTS_FILE);
                    }
                    screen = Screen::Menu;
                }
            }
            Screen::HumanVsAi => {
                if vs_human_alive || vs_ai_alive {
                    let gs = (7.0 + vs_human_score.max(vs_ai_score) as f32 / 100.0).min(17.0);
                    vs_ground_offset = (vs_ground_offset + gs * 60.0 * dt) % constants::GROUND_W;
                    vs_enemies.update(dt, gs * 60.0);

                    vs_human_player.update(dt);

                    // AI decision
                    if let Some(ref net) = vs_ai_network {
                        let feats = ai_features_at(
                            vs_ai_player.rect().x,
                            vs_ai_player.pos_y(),
                            vs_ai_player.vel_y(),
                            vs_ai_player.on_ground,
                            vs_ai_score,
                            &vs_enemies,
                        );
                        let jump = net.predict(&feats);
                        vs_ai_player.update_with_jump(dt, jump);
                    }

                    anim_timer += dt;
                    if anim_timer >= 0.1 {
                        anim_frame = !anim_frame;
                        anim_timer -= 0.1;
                    }
                    vs_score_timer += dt;
                    if vs_score_timer >= 0.1 {
                        if vs_human_alive {
                            vs_human_score += 1;
                        }
                        if vs_ai_alive {
                            vs_ai_score += 1;
                        }
                        vs_score_timer -= 0.1;
                    }

                    let hr = Rect::new(
                        vs_human_player.rect().x + 10.0,
                        vs_human_player.rect().y + 10.0,
                        vs_human_player.rect().w - 20.0,
                        vs_human_player.rect().h - 20.0,
                    );
                    if vs_human_alive && vs_enemies.check_collision(&hr) {
                        vs_human_alive = false;
                    }
                    // AI player rect
                    let ai_rect = {
                        let r = vs_ai_player.rect();
                        Rect::new(r.x + 10.0, r.y + 10.0, r.w - 20.0, r.h - 20.0)
                    };
                    if vs_ai_alive && vs_enemies.check_collision(&ai_rect) {
                        vs_ai_alive = false;
                    }

                    // Determine winner when someone dies
                    if !vs_human_alive || !vs_ai_alive {
                        if !vs_human_alive && vs_ai_alive {
                            vs_winner = 2; // AI Wins
                        } else if vs_human_alive && !vs_ai_alive {
                            vs_winner = 1; // Human Wins
                        } else if !vs_human_alive && !vs_ai_alive {
                            // Both died - check scores
                            if vs_human_score > vs_ai_score {
                                vs_winner = 1;
                            } else if vs_ai_score > vs_human_score {
                                vs_winner = 2;
                            } else {
                                vs_winner = 0; // Actual Draw
                            }
                        }
                    }
                }

                // Render (only while at least one is alive)
                if vs_human_alive || vs_ai_alive {
                    draw_ground(&sprite, vs_ground_offset);
                    vs_enemies.draw(&sprite);

                    // Human dinosaur
                    let h_frame = if vs_human_player.on_ground {
                        if anim_frame {
                            constants::PLAYER_FRAME1_X
                        } else {
                            constants::PLAYER_FRAME2_X
                        }
                    } else {
                        constants::PLAYER_JUMP_X
                    };
                    let hp = DrawTextureParams {
                        source: Some(Rect::new(
                            h_frame,
                            0.,
                            constants::PLAYER_SRC_W,
                            constants::PLAYER_SRC_H,
                        )),
                        dest_size: Some(Vec2::new(
                            constants::PLAYER_SIZE.0,
                            constants::PLAYER_SIZE.1,
                        )),
                        ..Default::default()
                    };
                    draw_texture_ex(
                        &sprite,
                        screen_width() * 0.10 + 60.0,
                        vs_human_player.pos_y(),
                        WHITE,
                        hp,
                    );
                    draw_text(
                        "YOU",
                        screen_width() * 0.10 + 60.0,
                        vs_human_player.pos_y() - 10.,
                        12.,
                        BLUE,
                    );

                    // AI dinosaur
                    let a_frame = if vs_ai_player.on_ground {
                        if anim_frame {
                            constants::PLAYER_FRAME1_X
                        } else {
                            constants::PLAYER_FRAME2_X
                        }
                    } else {
                        constants::PLAYER_JUMP_X
                    };
                    let ap = DrawTextureParams {
                        source: Some(Rect::new(
                            a_frame,
                            0.,
                            constants::PLAYER_SRC_W,
                            constants::PLAYER_SRC_H,
                        )),
                        dest_size: Some(Vec2::new(
                            constants::PLAYER_SIZE.0,
                            constants::PLAYER_SIZE.1,
                        )),
                        ..Default::default()
                    };
                    draw_texture_ex(
                        &sprite,
                        screen_width() * 0.10,
                        vs_ai_player.pos_y(),
                        WHITE,
                        ap,
                    );
                    draw_text(
                        "AI",
                        screen_width() * 0.10,
                        vs_ai_player.pos_y() - 10.,
                        12.,
                        RED,
                    );

                    let st = format!("You: {}  AI: {}", vs_human_score, vs_ai_score);
                    draw_text(&st, 15., 30., 22., BLACK);

                    if show_hitboxes {
                        let hr = vs_human_player.rect();
                        draw_rectangle_lines(hr.x, hr.y, hr.w, hr.h, 2., BLUE);
                        let ar = vs_ai_player.rect();
                        draw_rectangle_lines(ar.x, ar.y, ar.w, ar.h, 2., RED);
                        for hr in vs_enemies.hitboxes() {
                            draw_rectangle_lines(
                                hr.x,
                                hr.y,
                                hr.w,
                                hr.h,
                                2.,
                                Color::new(0.5, 0.0, 0.0, 1.0),
                            );
                        }
                    }
                } else {
                    // Game over — show winner
                    let msg = match vs_winner {
                        1 => "YOU WIN!",
                        2 => "AI WINS!",
                        _ => "DRAW!",
                    };
                    let mw = measure_text(msg, None, 40u16, 1.).width;
                    draw_text(
                        msg,
                        screen_width() / 2. - mw / 2.,
                        screen_height() / 2. - 20.,
                        40.,
                        BLACK,
                    );
                    let st = format!("You: {}  AI: {}", vs_human_score, vs_ai_score);
                    let sw = measure_text(&st, None, 25u16, 1.).width;
                    draw_text(
                        &st,
                        screen_width() / 2. - sw / 2.,
                        screen_height() / 2. + 30.,
                        25.,
                        DARKGRAY,
                    );
                    let restart = "Press SPACE to replay or ESC for menu";
                    let rw = measure_text(restart, None, 16u16, 1.).width;
                    draw_text(
                        restart,
                        screen_width() / 2. - rw / 2.,
                        screen_height() / 2. + 70.,
                        16.,
                        DARKGRAY,
                    );

                    if is_key_pressed(KeyCode::Space) {
                        vs_human_player = Player::new();
                        vs_ai_player = Player::new();
                        vs_enemies = Enemies::new();
                        vs_human_score = 0;
                        vs_ai_score = 0;
                        vs_human_alive = true;
                        vs_ai_alive = true;
                        vs_score_timer = 0.0;
                        vs_ground_offset = 0.0;
                        vs_winner = 0;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    screen = Screen::Menu;
                }
                if show_hitboxes {
                    draw_text("HITBOXES ON (F1)", 15., 55., 15., DARKGRAY);
                }
            }
        }

        next_frame().await;
    }
}

fn draw_nn(network: &DinoNet<B>) {
    let weights = network.get_weights();

    let layer1_w = &weights[0..INPUT * HIDDEN];
    let layer2_w = &weights[INPUT * HIDDEN + HIDDEN..INPUT * HIDDEN + HIDDEN + HIDDEN];

    let center_x = screen_width() / 2.0;
    let center_y = 80.0;
    let node_radius = 5.0;
    let layer_gap = 120.0;
    let node_gap = 15.0;

    let input_nodes = (0..INPUT)
        .map(|i| {
            Vec2::new(
                center_x - layer_gap,
                center_y + (i as f32 - (INPUT as f32 - 1.0) / 2.0) * node_gap,
            )
        })
        .collect::<Vec<_>>();

    let hidden_nodes = (0..HIDDEN)
        .map(|i| {
            Vec2::new(
                center_x,
                center_y + (i as f32 - (HIDDEN as f32 - 1.0) / 2.0) * node_gap,
            )
        })
        .collect::<Vec<_>>();

    let output_nodes = [Vec2::new(center_x + layer_gap, center_y)];

    // Connections Layer 1
    for i in 0..INPUT {
        for j in 0..HIDDEN {
            let w = layer1_w[i * HIDDEN + j];
            let color = if w > 0.0 {
                Color::new(0.0, 0.0, 1.0, w.abs().min(1.0) * 0.5)
            } else {
                Color::new(1.0, 0.0, 0.0, w.abs().min(1.0) * 0.5)
            };
            draw_line(
                input_nodes[i].x,
                input_nodes[i].y,
                hidden_nodes[j].x,
                hidden_nodes[j].y,
                (w.abs() * 2.0).min(3.0),
                color,
            );
        }
    }

    // Connections Layer 2
    for j in 0..HIDDEN {
        let w = layer2_w[j];
        let color = if w > 0.0 {
            Color::new(0.0, 0.0, 1.0, w.abs().min(1.0) * 0.5)
        } else {
            Color::new(1.0, 0.0, 0.0, w.abs().min(1.0) * 0.5)
        };
        draw_line(
            hidden_nodes[j].x,
            hidden_nodes[j].y,
            output_nodes[0].x,
            output_nodes[0].y,
            (w.abs() * 2.0).min(3.0),
            color,
        );
    }

    // Nodes
    for node in input_nodes
        .iter()
        .chain(hidden_nodes.iter())
        .chain(output_nodes.iter())
    {
        draw_circle(node.x, node.y, node_radius, DARKGRAY);
        draw_circle_lines(node.x, node.y, node_radius, 1.0, BLACK);
    }
}

fn draw_ground(texture: &Texture2D, offset: f32) {
    let gy = constants::ground_y();
    let gp = || DrawTextureParams {
        source: Some(Rect::new(
            0.,
            constants::GROUND_SRC_Y,
            constants::GROUND_W,
            constants::GROUND_H,
        )),
        dest_size: Some(Vec2::new(constants::GROUND_W, constants::GROUND_H)),
        ..Default::default()
    };
    draw_texture_ex(texture, -offset, gy - 24., WHITE, gp());
    draw_texture_ex(
        texture,
        -offset + constants::GROUND_W,
        gy - 24.,
        WHITE,
        gp(),
    );
}
