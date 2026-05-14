# Dino Game

An **endless runner** inspired by the Chrome no-internet dinosaur game, built entirely in **Rust**. Features human play, a **Genetic Algorithm** that trains 100 AI agents simultaneously to teach itself how to play, a **neural network** defined with the Burn deep-learning framework, and a Human vs. AI competitive mode. All sprites are rendered from a single sprite sheet using pixel-precise texture cropping.

---

## Table of Contents

- [Overview](#overview)
- [Screens & Modes](#screens--modes)
  - [Menu](#menu)
  - [Human Play](#human-play)
  - [AI Training](#ai-training)
  - [Human vs AI](#human-vs-ai)
- [Controls](#controls)
- [Architecture](#architecture)
  - [Project Structure](#project-structure)
  - [Module Breakdown](#module-breakdown)
  - [Game Loop & Screen Machine](#game-loop--screen-machine)
- [Sprite Sheet Cropping — Every Pixel Explained](#sprite-sheet-cropping--every-pixel-explained)
  - [How `draw_texture_ex` Crops Textures](#how-draw_texture_ex-crops-textures)
  - [Player Sprite Frames](#player-sprite-frames)
  - [Obstacle Sprites](#obstacle-sprites)
  - [Ground Texture](#ground-texture)
  - [Putting It All Together — The Render Pipeline](#putting-it-all-together--the-render-pipeline)
- [Physics System](#physics-system)
  - [Gravity & Jump Mechanics](#gravity--jump-mechanics)
  - [Ground Clamping](#ground-clamping)
  - [Score-Based Speed Scaling](#score-based-speed-scaling)
- [Collision Detection](#collision-detection)
  - [Hitbox Design](#hitbox-design)
  - [AABB Overlap](#aabb-overlap)
  - [Debug Visualization](#debug-visualization)
- [Obstacle System](#obstacle-system)
  - [Obstacle Types](#obstacle-types)
  - [Spawn Cycle — Alternating Small & Big](#spawn-cycle--alternating-small--big)
  - [Scroll & Wrapping Logic](#scroll--wrapping-logic)
- [Ground Scrolling](#ground-scrolling)
- [Animation System](#animation-system)
  - [Player Running Animation](#player-running-animation)
- [AI System](#ai-system)
  - [Neural Network (Burn)](#neural-network-burn)
  - [Input Features (7 Dimensions)](#input-features-7-dimensions)
  - [Genetic Algorithm](#genetic-algorithm)
  - [Population & Parallel Evaluation](#population--parallel-evaluation)
  - [Brain Persistence](#brain-persistence)
- [Scoring System](#scoring-system)
- [Constants & Configuration](#constants--configuration)
- [Building & Running](#building--running)
- [Dependencies](#dependencies)
- [License](#license)

---

## Overview

This project is a complete, polished clone of the Chrome dino game with three major additions beyond the original:

1. **Full game** with ground scrolling, obstacle spawning, gravity physics, collision detection, and scoring
2. **AI that learns to play** via a Genetic Algorithm evolving a neural network — 100 agents run simultaneously in each generation
3. **Human vs AI mode** where you compete against the best trained brain

Everything is rendered from a single `sprite.png` sheet using macroquad's `draw_texture_ex` with sub-rectangle cropping. No separate image files for each frame — every sprite is sliced out of the atlas at exact pixel coordinates.

---

## Screens & Modes

### Menu

Three options displayed as a numbered list. The menu shows a green hint if a trained AI brain exists on disk (loaded on startup), indicating that the AI is ready for Human vs AI mode.

| Key | Mode |
|-----|------|
| `1` | Human Play |
| `2` | AI Training |
| `3` | Human vs AI |

### Human Play

The classic endless runner. Press SPACE to jump over cacti. The game speed increases as your score climbs. Crash into an obstacle and return to the menu. Your high score is tracked for the session.

### AI Training

Opens a screen where 100 AI agents (each with their own neural network) are evaluated in parallel against the same shared obstacles. All agents start at the same X position and are rendered at full dinosaur size. The best-performing agent (highest current score) is rendered at full opacity and labelled "BEST"; the other 99 are rendered at 20% opacity so you can see the full population without visual clutter.

**What you see:**
- The scrolling game world with ground and obstacles
- 100 dinosaurs overlaid at the same X position — the best one stands out at full brightness
- A live neural network visualization (CodeBullet-style) showing the best agent's brain with coloured connection lines (blue = positive weight, red = negative weight)
- HUD showing generation number, alive count, best fitness, and top score
- When a generation ends, a large animated "Generation X evolved!" message appears

**How training works behind the scenes:**
- 100 agents = 100 individual neural networks, each with randomly initialised weights
- Each frame, every alive agent runs its network forward (7 inputs → 12 hidden neurons → 1 output) to decide whether to jump
- Agents share identical obstacle scroll state — they face the exact same challenges at the same time
- When an agent collides (or reaches score 5000), it dies and its final score is recorded as fitness
- Once all 100 agents are dead, the GA evolves the population: tournament selection, uniform crossover, Gaussian mutation, elitism
- The best network from each generation is auto-saved to `best_brain.json` on disk

### Human vs AI

You (blue label, offset right) and the AI (red label, at normal position) run the same obstacle course simultaneously. Both dinosaurs jump independently — you control yours with SPACE, the AI uses the best saved brain from training.

- Shared obstacles, shared ground, identical conditions
- When one of you crashes, the other is declared the winner
- If both crash simultaneously, the higher score wins
- Press SPACE to rematch with the same AI brain

If no brain has been trained yet, the game falls back to a quick 5-generation training session so you have something to play against.

---

## Controls

| Key | Action |
|-----|--------|
| `SPACE` | Jump (Human Play / Human vs AI) |
| `F1` | Toggle hitbox visualization |
| `ESC` | Return to menu |
| `1` / `2` / `3` | Menu selection |

---

## Architecture

### Project Structure

```
dino/
├── Cargo.toml
├── best_brain.json              # Persisted AI brain (auto-saved)
├── assets/
│   └── sprite.png               # Single sprite sheet (2404 × 130 px)
└── src/
    ├── main.rs                   # Entry point, screen machine, rendering
    ├── constants.rs              # All game constants, sprite coordinates
    ├── entity/
    │   ├── mod.rs                # Re-exports
    │   ├── player.rs             # Player struct, physics, drawing
    │   └── enemies.rs            # Obstacle manager, collision
    └── ai/
        ├── mod.rs                # Module re-exports
        ├── network.rs            # Burn neural network + feature computation
        ├── ga.rs                 # Genetic Algorithm (population, selection, crossover, mutation)
        ├── agent.rs              # AgentState, WorldAgent, factory functions
        └── simulation.rs         # Isolated game simulation for pre-training
```

### Module Breakdown

#### `main.rs` — Entry Point & Screen Machine

The game loop is a single infinite `loop { ... next_frame().await }` with a `Screen` enum driving the state:

```rust
enum Screen { Menu, HumanPlay, AiTrain, HumanVsAi }
```

Each screen has its own update logic and rendering. The `sprite` texture is loaded once at startup and shared across all screens. State variables for each screen are kept in `main()`'s stack and reused when switching screens.

Key responsibilities by screen:

| Screen | Updates | Renders |
|--------|---------|---------|
| Menu | Key polling for 1/2/3 | Title text, menu items, green hint if brain saved |
| HumanPlay | Physics, enemies, scoring, collision | Ground, enemies, player, score, hitboxes |
| AiTrain | 100 agents' physics, GA evolution, brain saving | Ground, obstacles, all 100 agents (best highlighted), NN visualization, HUD |
| HumanVsAi | Two players (human+AI), shared enemies, scoring, winner detection | Ground, obstacles, both dinos with labels, scores, winner announcement |

#### `constants.rs` — Single Source of Truth

Every tunable value — from gravity to sprite pixel coordinates — lives here. No magic numbers anywhere else.

#### `entity/player.rs` — The Dinosaur

A lightweight struct holding only position (Y varies), vertical velocity, and ground-contact flag. Physics are computed in `update_with_jump(dt, jump)` — the public `update(dt)` method reads keyboard input, while the AI path calls `update_with_jump` directly with the network's decision.

#### `entity/enemies.rs` — Obstacle Manager

Manages two obstacles (one small, one big) that alternate. Each obstacle has its own scroll position, width multiplier (1×–3×), and sprite variant. The `check_collision` method uses macroquad's `Rect::overlaps` with properly inset rectangles.

#### `ai/network.rs` — Neural Network (Burn)

A 2-layer feedforward network defined with the Burn deep-learning framework:

```
Input(7) → Linear(7→12) → ReLU → Linear(12→1) → Output(1)
```

The network is generic over `Backend` and uses `burn::backend::NdArray<f32>` (CPU). Weights are stored as a flat `Vec<f32>` (109 floats) for easy GA manipulation. The `from_weights` / `get_weights` pair converts between the flat gene representation and Burn's `Record`-based parameter storage.

#### `ai/ga.rs` — Genetic Algorithm

Population of 100 individuals, each with a `Vec<f32>` of 109 genes and a fitness score.

| Parameter | Value |
|-----------|-------|
| Population | 100 |
| Elite count | 2 |
| Crossover rate | 70% |
| Mutation rate | 10% per gene |
| Mutation stddev | 0.2 |
| Tournament size | 3 |

**Evolution pipeline:**

1. **Evaluate** — each agent plays the game until death or score cap (5000), recording its final score as fitness
2. **Sort** — descending by fitness
3. **Elite** — top 2 individuals survive unchanged
4. **Selection** — tournament of 3, pick the best
5. **Crossover** — uniform: each gene has a 50% chance of coming from either parent (70% of offspring use crossover)
6. **Mutation** — Gaussian noise: each gene has a 10% chance of `+= rand(-1,1) * 0.2`, clamped to ±5

#### `ai/agent.rs` — Agent Types

Two agent representations:

- **`AgentState`** — used for isolated pre-training (each agent has its own `Simulation` with independent obstacles). Created by `create_agents()`.
- **`WorldAgent`** — used in the AiTrain shared-world view. All 100 agents share one `Enemies` instance. Each has its own physics state (y, vy, on_ground) because they jump independently, but obstacle scroll is shared.

#### `ai/simulation.rs` — Isolated Game Sim

A self-contained game simulation with its own player physics, obstacle management, scoring, and collision. Used for the quick 5-generation fallback training in Human vs AI mode (when no saved brain exists). This simulation is NOT used in the AiTrain screen — that uses the shared-world approach with WorldAgents.

### Game Loop & Screen Machine

```
loop {
    let dt = get_frame_time();
    clear_background(WHITE);
    
    // Global input (works in any screen)
    if is_key_pressed(F1) { show_hitboxes = !show_hitboxes; }
    
    match screen.clone() {
        Screen::Menu     => { /* render menu, handle 1/2/3 */ }
        Screen::HumanPlay => { /* update + render game */ }
        Screen::AiTrain   => { /* update 100 agents + evolve + render */ }
        Screen::HumanVsAi => { /* update human + AI + render */ }
    }
    
    next_frame().await;
}
```

The `screen.clone()` pattern avoids borrow-checker issues with the match, since the screen can be reassigned inside match arms.

---

## Sprite Sheet Cropping — Every Pixel Explained

### How `draw_texture_ex` Crops Textures

All sprites come from a single file: `assets/sprite.png` (2404 pixels wide × 130 pixels tall). The key macroquad function is:

```rust
draw_texture_ex(
    &texture,           // The loaded sprite sheet
    dest_x,            // Screen X to draw at
    dest_y,            // Screen Y to draw at
    WHITE,             // Colour tint (multiply)
    DrawTextureParams {
        source: Some(Rect::new(src_x, src_y, src_w, src_h)),
        dest_size: Some(Vec2::new(dest_w, dest_h)),
        ..Default::default()
    },
);
```

The `source` parameter defines a **sub-rectangle** of the texture in pixel coordinates. Only the pixels within that rectangle are sampled. Without `source`, the entire texture would be drawn. With `source`, we crop out exactly the sprite we need.

The `dest_size` parameter controls how large that cropped region appears on screen. If `dest_size` differs from the source dimensions, the sprite is scaled. We use `FilterMode::Nearest` so that scaling produces crisp pixel edges rather than blurry interpolation.

This is how every game element — the dinosaur, the cacti, the ground — is sliced out of the single texture atlas.

### Player Sprite Frames

The dinosaur occupies the top row of the sprite sheet (Y = 0). The original JavaScript prototype defines these coordinates:

```javascript
// char = 89x94          -> Destination rectangle on canvas
// char 1 @ 1514         -> Running frame 1 source X
// char 2 @ 1603         -> Running frame 2 source X
```

The source sprite is 88 pixels wide by 94 pixels tall (the JS uses 88 as the source crop width). The destination is 89 pixels wide (slightly stretched for visual proportions). The constants:

| Constant | Value | Description |
|----------|-------|-------------|
| `PLAYER_SRC_W` | `88.0` | Source crop width (matches JS: `ctx.drawImage(..., 88, 94, ...)`) |
| `PLAYER_SRC_H` | `94.0` | Source crop height (all three frames are 94 px tall) |
| `PLAYER_SIZE.0` | `89.0` | Destination width on screen (89 = 88 + 1 px stretch from JS) |
| `PLAYER_SIZE.1` | `94.0` | Destination height on screen (1:1 with source) |
| `PLAYER_JUMP_X` | `1338.0` | Sprite X for jump/standing pose |
| `PLAYER_FRAME1_X` | `1514.0` | Sprite X for running frame 1 (left leg forward) |
| `PLAYER_FRAME2_X` | `1602.0` | Sprite X for running frame 2 (right leg forward) |

**Why 1338, 1514, and 1602?** These are the exact X offsets in the sprite sheet where each 88×94 dinosaur frame begins. The original sprite sheet was designed by Google's artists with these specific positions. The JavaScript prototype uses the same values. Between frame 1 (1514) and frame 2 (1602) there are 88 pixels of difference, which is exactly the width of one frame — confirming they are laid out sequentially.

The drawing call:
```rust
draw_texture_ex(&sprite, dest_x, dest_y, WHITE, DrawTextureParams {
    source: Some(Rect::new(PLAYER_FRAME1_X, 0.0, 88.0, 94.0)),
    dest_size: Some(Vec2::new(89.0, 94.0)),
    ..Default::default()
});
```

This reads 88×94 pixels starting at (1514, 0) from the texture, then renders them at 89×94 pixels on screen. The 1-pixel horizontal stretch is imperceptible and matches the original JS behaviour.

### Obstacle Sprites

Obstacles also live in the top portion of the sprite sheet, at Y = 2 (the first two rows have some transparent separation). There are two types, each with variants:

#### Small Cacti

| Constant | Value | Description |
|----------|-------|-------------|
| `SMALL_W` | `34.0` | Base width of one small cactus |
| `SMALL_H` | `70.0` | Height of small cactus |
| `SMALL_PICS[0]` | `446.0` | Sprite X for small cactus variant A |
| `SMALL_PICS[1]` | `548.0` | Sprite X for small cactus variant B |
| `SMALL_INIT_SCROLL` | `-100.0` | Initial scroll offset (starts off-screen right) |

The two variants are at X = 446 and X = 548. The gap between them is 102 pixels, which is exactly 3 × 34 (three times the base width). This is because the JavaScript code draws them with a width multiplier:

```javascript
// multiS = random 1, 2, or 3
// picS = 446 or 548
// Source width = obsS.w * multiS  (34, 68, or 102)
```

When `multi = 1`, only the first 34 pixels are drawn (one cactus). When `multi = 2`, 68 pixels are drawn (two cacti side by side). When `multi = 3`, 102 pixels are drawn (three cacti). The sprite sheet has groups of 1, 2, and 3 cacti packed at these X positions, so the width multiplier directly selects how many appear.

The drawing call:
```rust
draw_texture_ex(&sprite, screen_x, ground_y - 70.0, WHITE, DrawTextureParams {
    source: Some(Rect::new(pic, 2.0, SMALL_W * multi, SMALL_H)),
    dest_size: Some(Vec2::new(SMALL_W * multi, SMALL_H)),
    ..Default::default()
});
```

The source Y is 2.0 (not 0) because the obstacle sprites are positioned at Y = 2 in the sheet, one row below the player frames.

#### Big Cacti

| Constant | Value | Description |
|----------|-------|-------------|
| `BIG_W` | `49.0` | Base width of one big cactus |
| `BIG_H` | `100.0` | Height of big cactus |
| `BIG_PICS[0]` | `652.0` | Sprite X for big cactus variant A |
| `BIG_PICS[1]` | `802.0` | Sprite X for big cactus variant B |
| `BIG_INIT_SCROLL` | `-200.0` | Initial scroll offset (starts further off-screen right) |

The two big cactus variants are at X = 652 and X = 802. The gap is 150 pixels, which is approximately 3 × 49 (three times the base width). Same logic as small cacti: `multi = 1, 2, 3` selects how many cacti appear in the group.

Source Y = 2.0, same as small cacti — they share the same row in the sprite sheet.

#### Effective Obstacle Widths

| Multiplier | Small Width | Big Width |
|-----------|-------------|-----------|
| 1× | 34 px | 49 px |
| 2× | 68 px | 98 px |
| 3× | 102 px | 147 px |

The destination Y for obstacles is always `ground_y() - obs.h`, placing their base exactly on the ground line. This is computed dynamically each frame because `ground_y()` depends on `screen_height()`.

### Ground Texture

| Constant | Value | Description |
|----------|-------|-------------|
| `GROUND_SRC_Y` | `104.0` | Y position of ground strip in sprite sheet |
| `GROUND_H` | `18.0` | Height of the ground strip |
| `GROUND_W` | `2404.0` | Width of the ground strip |

The ground is a single 2404×18 pixel strip located at Y = 104 in the sprite sheet. It is rendered in two copies side by side for seamless wrapping:

```rust
fn draw_ground(texture: &Texture2D, offset: f32) {
    let gy = constants::ground_y();
    let params = || DrawTextureParams {
        source: Some(Rect::new(0.0, GROUND_SRC_Y, GROUND_W, GROUND_H)),
        dest_size: Some(Vec2::new(GROUND_W, GROUND_H)),
        ..Default::default()
    };
    draw_texture_ex(texture, -offset, gy - 24.0, WHITE, params());
    draw_texture_ex(texture, -offset + GROUND_W, gy - 24.0, WHITE, params());
}
```

The offset cycles from 0 to 2404 (wrapping with modulo). Two copies are drawn because when the first copy scrolls off the left edge (offset > 0), the second copy is already visible on the right at `-offset + 2404`. The Y position is `gy - 24` (24 pixels above the ground line), matching the original JS: `plat.y - 24`.

### Putting It All Together — The Render Pipeline

Rendering follows strict back-to-front order:

1. **Clear** — `clear_background(WHITE)` erases everything
2. **Ground** — Two tiled copies of the 2404×18 strip at Y = `ground_y() - 24`
3. **Obstacles** — Both small and big obstacles drawn at their current scroll positions. The inactive obstacle is typically off-screen (either entering from the right at initial scroll, or already exited to the left)
4. **Player / Agents** — Human Play: one dinosaur. AI Training: 100 dinosaurs (best highlighted). Human vs AI: two dinosaurs with labels
5. **UI** — Score text, menu text, game over overlay, hitbox outlines, NN visualization

Each step calls `draw_texture_ex` with its own `source` rectangle, cropping exactly the right pixels from the sheet.

---

## Physics System

### Gravity & Jump Mechanics

The player moves vertically with velocity-based physics, converted from the JavaScript prototype's per-frame values to per-second values:

| JS (per frame) | Rust (per second) | Derivation |
|----------------|-------------------|------------|
| `grav = 0.6` | `GRAVITY = 2200.0` | 0.6 × 60² ≈ 2160, rounded to 2200 |
| `jump = 15` | `JUMP_VEL = 900.0` | 15 × 60 = 900 |

Each frame:
```rust
vel_y += GRAVITY * dt;   // Apply gravity (increases downward velocity)
pos.y += vel_y * dt;     // Update position by velocity × time
```

This produces a jump arc with:
- Time to apex: `900 / 2200 ≈ 0.41 seconds`
- Max jump height: `900² / (2 × 2200) ≈ 184 pixels`
- Total jump duration: `2 × 0.41 ≈ 0.82 seconds`

### Ground Clamping

After applying gravity and updating position, the player is clamped to the ground:

```rust
let ground = ground_y() - PLAYER_SIZE.1;
if pos.y >= ground {
    pos.y = ground;
    vel_y = 0.0;
    on_ground = true;
}
```

This prevents the player from falling through the floor. `on_ground` is set to `true` only when the player is resting on the ground, which gates the jump input (no double jumps).

### Score-Based Speed Scaling

Game speed affects obstacle scroll rate and ground scroll rate. It scales with score:

```rust
let gs = (7.0 + score as f32 / 100.0).min(17.0);
```

- Starts at 7.0 when score = 0
- Increases by 1.0 for every 100 points
- Caps at 17.0 (when score ≥ 1000)

This speed is the number of pixels per FRAME at 60 FPS in the original JS. In Rust with variable delta time, it's converted to per-second: `gs * 60.0 * dt`.

---

## Collision Detection

### Hitbox Design

All hitboxes are axis-aligned bounding boxes (AABB). To make the game feel fair, both player and obstacle hitboxes are **inset** slightly:

**Player hitbox** (from `Player::rect()`):
```rust
Rect::new(self.pos.x + 5.0, self.pos.y + 5.0, 79.0, 84.0)
```
- 5 px inset on each side horizontally (89 → 79)
- 5 px inset from top, leaving bottom inset (94 → 84)

**Obstacle hitbox** (from `Enemies::check_collision()`):
```rust
Rect::new(obs.screen_x() + 5.0, ground_y() - obs.h + 5.0, obs.w() - 10.0, obs.h - 10.0)
```
- 5 px inset on each side
- 5 px inset from top and bottom

This 5-pixel forgiveness means visual sprites can slightly overlap before a collision registers, which dramatically improves the feel of the game — the AI also learns with these exact inset values so its skill transfers to human play.

### AABB Overlap

Collision uses macroquad's built-in `Rect::overlaps()`:

```rust
pub fn overlaps(&self, other: &Rect) -> bool {
    self.x < other.x + other.w
        && self.x + self.w > other.x
        && self.y < other.y + other.h
        && self.y + self.h > other.y
}
```

This checks both axes simultaneously: two rectangles overlap if and only if their projections overlap on both X and Y axes.

### Debug Visualization

Pressing `F1` toggles hitbox rendering:
- **Green outline** — player's collision rect
- **Red outline** — each obstacle's collision rect
- **Label** — "HITBOXES ON (F1)" appears on screen

These use `draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, color)` to draw unfilled rectangles.

---

## Obstacle System

### Obstacle Types

Two types of obstacles, each rendered from the sprite sheet:

| Type | Base Width | Height | Multiplier Range | Effective Widths | Sprite Y |
|------|-----------|--------|-----------------|------------------|----------|
| Small cactus | 34 px | 70 px | 1–3× | 34, 68, 102 px | 2 |
| Big cactus | 49 px | 100 px | 1–3× | 49, 98, 147 px | 2 |

Each type has 2 visual variants (sprite X positions 446/548 for small, 652/802 for big) selected randomly on each activation.

### Spawn Cycle — Alternating Small & Big

The `Enemies` manager alternates between small and big obstacles. The spawn logic is deterministic in its alternation but randomized in its specifics:

```
Initialize: small activates immediately
Loop:
  while small is active:
    small scrolls left at game_speed
    when small finishes → big activates with random multi + variant
  while big is active:
    big scrolls left at game_speed  
    when big finishes → small activates with random multi + variant
```

This guarantees you never see two small cacti in a row or two big cacti in a row — they strictly alternate.

### Scroll & Wrapping Logic

Each obstacle has a `scroll` value that starts negative (off-screen right) and increases over time. The screen position is computed as:

```rust
fn screen_x(&self) -> f32 {
    screen_width() - self.scroll
}
```

When `scroll = -100`, screen position = `screen_width() + 100` (100 px off-screen right). As scroll increases, the obstacle moves left across the screen.

An obstacle finishes when:
```rust
fn finished(&self) -> bool {
    self.scroll > screen_width() + self.w() * 3.0
}
```

The `+ w() * 3.0` extra distance ensures the obstacle scrolls well past the left edge before it resets, creating a natural gap before the next obstacle appears. The gap is proportional to the obstacle's width — wider obstacles leave a larger gap.

When finished, the obstacle calls `activate()`:
```rust
fn activate(&mut self) {
    self.multi = random(1..=3);
    self.pic = random_pic();
    self.scroll = self.init_scroll;  // Reset to off-screen right
}
```

---

## Ground Scrolling

The ground is a 2404×18 pixel strip from the sprite sheet. It scrolls by incrementing an offset and using modular arithmetic:

```rust
ground_offset = (ground_offset + gs * 60.0 * dt) % GROUND_W;
```

The draw function renders two copies:
```rust
draw_texture_ex(texture, -offset, gy - 24.0, WHITE, ground_params());
draw_texture_ex(texture, -offset + GROUND_W, gy - 24.0, WHITE, ground_params());
```

When offset = 0: first copy at x=0, second copy at x=2404 (off-screen).
When offset = 2000: first copy at x=-2000 (off-screen left), second copy at x=404 (visible from 404 to 800 on an 800-pixel screen).

At any offset value, exactly one of the two copies is guaranteed to cover the viewport (0–800 pixels). The modulo operation keeps offset in [0, 2404) so it never overflows.

---

## Animation System

### Player Running Animation

The dinosaur has three visual states:

| State | Sprite Source X | Description |
|-------|----------------|-------------|
| Jumping | 1338 | Static pose, legs tucked — used whenever `on_ground == false` |
| Run frame 1 | 1514 | Left leg forward |
| Run frame 2 | 1602 | Right leg forward |

The running animation toggles every 0.1 seconds (10 FPS animation rate):

```rust
anim_timer += dt;
if anim_timer >= 0.1 {
    anim_frame = !anim_frame;
    anim_timer -= 0.1;
}
```

Frame selection logic:
```rust
let src_x = if !on_ground {
    PLAYER_JUMP_X       // 1338 — jump pose
} else if anim_frame {
    PLAYER_FRAME1_X     // 1514 — running frame 1
} else {
    PLAYER_FRAME2_X     // 1602 — running frame 2
};
```

This runs at 10 Hz (5 complete cycles per second), which matches the original JS `frameInterval > 5` at 60 FPS (5 frames between toggles = ~12 FPS animation rate).

---

## AI System

### Neural Network (Burn)

The AI uses a small feedforward neural network defined with the **Burn** deep learning framework (v0.16, ndarray backend):

```rust
#[derive(Module, Debug)]
pub struct DinoNet<B: Backend> {
    fc1: nn::Linear<B>,  // 7 → 12
    fc2: nn::Linear<B>,  // 12 → 1
}
```

**Architecture:**
```
Input(7) → Linear(7→12) → ReLU → Linear(12→1) → Output(1)
```

**Total parameters:** `7×12 + 12 + 12×1 + 1 = 109`

Weights are stored as a flat `Vec<f32>` of 109 values, making GA operations trivial. The `from_weights` method constructs a Burn network from this flat vector by splitting it into the four parameter groups and loading them via Burn's `Record` system:

```rust
let (w1, r) = weights.split_at(INPUT * HIDDEN);   // 84 values → fc1.weight [7, 12]
let (b1, r) = r.split_at(HIDDEN);                  // 12 values → fc1.bias [12]
let (w2, b2) = r.split_at(HIDDEN);                 // 12 values → fc2.weight [12, 1]
// b2 = 1 value → fc2.bias [1]
```

The weight shapes are `[d_in, d_out]` because Burn's `Linear::forward` computes `x @ W + b` (no implicit transpose).

The `get_weights` method extracts all four parameter tensors, flattens them, and concatenates:

```rust
weights.extend(w1_flat);   // 84 values
weights.extend(b1_flat);   // 12 values
weights.extend(w2_flat);   // 12 values
weights.extend(b2_flat);   // 1 value
// Total: 109 values
```

### Input Features (7 Dimensions)

The network receives 7 normalized features each frame:

| Index | Feature | Formula | Range | Purpose |
|-------|---------|---------|-------|---------|
| 0 | Obstacle distance | `(obs_x - player_right) / screen_width` | [0, 1] | How far to the next obstacle |
| 1 | Obstacle width | `obs_w / screen_width` | [0, 1] | How wide the obstacle is |
| 2 | Obstacle height | `obs_h / screen_height` | [0, 1] | How tall the obstacle is |
| 3 | Player height | `(ground_y - player_y) / 200` | [0, 1] | How high off the ground (0=ground) |
| 4 | Vertical velocity | `player_vy / 800` | [-1, 1] | Falling or rising |
| 5 | On ground | `if on_ground { 1.0 } else { 0.0 }` | {0, 1} | Binary ground contact |
| 6 | Game speed | `(game_speed - 7) / 10` | [0, 1] | Normalized speed |

The nearest obstacle is found by iterating both obstacles' hitboxes and selecting the one with the smallest non-negative distance from `player_right = agent_x + PLAYER_SIZE.0`.

### Genetic Algorithm

**Population:** 100 individuals, each with 109 floating-point genes initialised randomly in [-1, 1].

**Evaluation:** Each agent plays the game until collision or score 5000. The final score is the fitness.

**Selection:** Tournament selection with size 3 — pick 3 random individuals, keep the best.

**Crossover:** Uniform crossover at 70% probability. Each gene has a 50% chance of coming from either parent. If crossover doesn't happen, one parent is cloned.

**Mutation:** Gaussian mutation at 10% per gene. Mutated genes get `+= random(-1, 1) × 0.2`, clamped to [-5, 5].

**Elitism:** The top 2 individuals survive unchanged into the next generation.

**Evolution trigger:** After all 100 agents have died (or hit the score cap), `evolve()` is called automatically. This updates `best_fitness` and `avg_fitness` for display, then produces the next generation via selection → crossover → mutation.

### Population & Parallel Evaluation

In the AI Training screen, all 100 agents are evaluated **simultaneously** against the same shared obstacle course:

```
Frame loop:
  1. Update enemies (one shared Enemies instance)
  2. For each alive agent:
     a. Compute 7 features from agent's state + shared enemies
     b. Run network forward → get jump decision
     c. Apply physics (gravity, ground clamp, optional jump)
     d. Check collision against shared obstacles
     e. If collided or score ≥ 5000: mark dead, record fitness, increment death counter
  3. If all 100 dead → evolve GA, create new agents, reset enemies
```

This is the true genetic algorithm in action — 100 agents competing simultaneously, not sequentially.

### Brain Persistence

After each generation evolves, the best network is:
1. Saved to memory (`saved_ai_brain: Option<Vec<f32>>`)
2. Saved to disk (`best_brain.json` via `serde_json`)

On startup, `best_brain.json` is loaded from disk. On ESC exit from AI Training, the best brain is saved again (in case the user quit mid-generation).

The Human vs AI mode reads `saved_ai_brain` (memory) — if no brain exists, it falls back to a quick 5-generation training session using the isolated `Simulation` (each agent trains independently with its own obstacles, optimized for speed).

---

## Scoring System

Score increments by 1 every 0.1 seconds (10 points per second):

```rust
score_timer += dt;
if score_timer >= 0.1 {
    score += 1;
    score_timer -= 0.1;
}
```

This matches the original JS: `scoreInterval > 6` at 60 FPS ≈ 8.5 points/sec, but 10/sec is close enough and feels smooth.

In Human Play mode, a `high_score` variable persists for the session. In AI Training, scores are used as the fitness metric for evolution. In Human vs AI, both players have independent scores.

---

## Constants & Configuration

All tunable values are centralized in `src/constants.rs`:

| Category | Constant | Value | Description |
|----------|----------|-------|-------------|
| Physics | `GRAVITY` | 2200.0 | px/s² downward acceleration |
| Physics | `JUMP_VEL` | 900.0 | px/s initial upward velocity |
| Display | `GROUND_RATIO` | 0.85 | Ground line as fraction of screen height |
| Player | `PLAYER_SIZE` | (89., 94.) | Draw dimensions |
| Player | `PLAYER_SRC_W` | 88. | Source crop width in sprite |
| Player | `PLAYER_JUMP_X` | 1338. | Jump frame X in sprite |
| Player | `PLAYER_FRAME1_X` | 1514. | Run frame 1 X in sprite |
| Player | `PLAYER_FRAME2_X` | 1602. | Run frame 2 X in sprite |
| Ground | `GROUND_SRC_Y` | 104. | Ground strip Y in sprite |
| Ground | `GROUND_W` | 2404. | Ground strip width |
| Ground | `GROUND_H` | 18. | Ground strip height |
| Small obs | `SMALL_W` | 34. | Base width |
| Small obs | `SMALL_H` | 70. | Height |
| Small obs | `SMALL_PICS` | [446., 548.] | Sprite X positions |
| Small obs | `SMALL_INIT_SCROLL` | -100. | Initial scroll offset |
| Big obs | `BIG_W` | 49. | Base width |
| Big obs | `BIG_H` | 100. | Height |
| Big obs | `BIG_PICS` | [652., 802.] | Sprite X positions |
| Big obs | `BIG_INIT_SCROLL` | -200. | Initial scroll offset |
| GA | `POPULATION` | 100 | Agents per generation |
| GA | `ELITE` | 2 | Elite survivors |
| GA | `MUTATION_RATE` | 0.10 | Mutation probability per gene |
| GA | `MUTATION_STD` | 0.2 | Mutation noise magnitude |
| GA | `TOURNAMENT` | 3 | Tournament selection size |
| Network | `INPUT` | 7 | Input feature count |
| Network | `HIDDEN` | 12 | Hidden layer size |
| Network | `NUM_PARAMS` | 109 | Total trainable parameters |

---

## Building & Running

### Prerequisites

- **Rust** (minimum edition 2024) — install via [rustup](https://rustup.rs/)
- A supported graphics backend (macroquad supports Metal/Vulkan on macOS, DirectX on Windows, OpenGL on Linux)

### Commands

```bash
# Run in debug mode
cargo run

# Run in release mode (faster)
cargo run --release

# Build only
cargo build

# Check for compilation errors
cargo check

# Run clippy lints
cargo clippy
```

### Asset Setup

Place the sprite sheet at `assets/sprite.png` (2404 × 130 pixels, PNG format). The game loads it with `load_texture("assets/sprite.png")` and applies `FilterMode::Nearest` for pixel-art rendering.

---

## Dependencies

```toml
[dependencies]
macroquad = "0.4.14"     # 2D game framework
rand = "0.8"             # Random number generation
burn = { version = "0.16", default-features = false, features = ["ndarray"] }  # Neural network
serde = { version = "1.0", features = ["derive"] }   # Serialization
serde_json = "1.0"       # JSON for brain persistence
```

- **macroquad** — Cross-platform rendering, input, texture loading, math types
- **rand** — Random obstacle variants, GA initialization, mutation
- **burn** — Neural network definition and inference (CPU via ndarray backend)
- **serde + serde_json** — Persist best brain to disk as JSON

---

## License

This project is provided for educational and personal use. The sprite sheet (`assets/sprite.png`) is derived from Google's Chrome dino game and may be subject to Google's terms of service.

---

*Built with Rust, macroquad, and Burn. Inspired by the Chrome no-internet dinosaur game and its JavaScript prototype (`dinoscript.js`).*
