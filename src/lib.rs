use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, MouseEvent, TouchEvent, Window,
};

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

#[derive(Clone)]
struct Player {
    x: f64,
    y: f64,
    radius: f64,
    speed: f64,
    cooldown: f64,
}

#[derive(Clone)]
struct Bullet {
    x: f64,
    y: f64,
    vy: f64,
    radius: f64,
}

#[derive(Clone)]
struct Enemy {
    x: f64,
    y: f64,
    size: f64,
    vy: f64,
    hp: i32,
    kind: u8,
    phase: f64,
}

#[derive(Clone)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    max_life: f64,
    size: f64,
}

struct Game {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    player: Player,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    particles: Vec<Particle>,
    keys: [bool; 5], // left, right, up, down, shoot
    pointer_x: Option<f64>,
    pointer_y: Option<f64>,
    score: u32,
    best: u32,
    lives: i32,
    level: u32,
    spawn_timer: f64,
    elapsed: f64,
    fire_timer: f64,
    paused: bool,
    over: bool,
    flash: f64,
    shake: f64,
    seed: u32,
}

impl Game {
    fn width(&self) -> f64 { self.canvas.width() as f64 }
    fn height(&self) -> f64 { self.canvas.height() as f64 }

    fn random(&mut self) -> f64 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed as f64) / (u32::MAX as f64)
    }

    fn reset(&mut self) {
        self.player = Player {
            x: self.width() / 2.0,
            y: self.height() - 85.0,
            radius: 18.0,
            speed: 390.0,
            cooldown: 0.0,
        };
        self.bullets.clear();
        self.enemies.clear();
        self.particles.clear();
        self.keys = [false; 5];
        self.pointer_x = None;
        self.pointer_y = None;
        self.score = 0;
        self.lives = 3;
        self.level = 1;
        self.spawn_timer = 0.5;
        self.elapsed = 0.0;
        self.fire_timer = 0.0;
        self.paused = false;
        self.over = false;
        self.flash = 0.0;
        self.shake = 0.0;
    }

    fn spawn_enemy(&mut self) {
        let r = self.random();
        let kind = if r < 0.14 { 2 } else if r < 0.34 { 1 } else { 0 };
        let size = match kind { 2 => 25.0, 1 => 20.0, _ => 16.0 };
        let hp = match kind { 2 => 3, 1 => 2, _ => 1 };
        let speed = 95.0 + self.random() * 65.0 + self.level as f64 * 7.0;
        self.enemies.push(Enemy {
            x: 35.0 + self.random() * (self.width() - 70.0),
            y: -35.0,
            size,
            vy: speed,
            hp,
            kind,
            phase: self.random() * PI * 2.0,
        });
    }

    fn shoot(&mut self) {
        if self.fire_timer > 0.0 || self.over { return; }
        self.bullets.push(Bullet {
            x: self.player.x,
            y: self.player.y - 23.0,
            vy: -650.0,
            radius: 4.0,
        });
        self.fire_timer = 0.16;
        self.burst(self.player.x, self.player.y - 20.0, 3, 70.0);
    }

    fn burst(&mut self, x: f64, y: f64, count: usize, speed: f64) {
        for _ in 0..count {
            let a = self.random() * PI * 2.0;
            let s = speed * (0.45 + self.random() * 0.7);
            self.particles.push(Particle {
                x, y,
                vx: a.cos() * s,
                vy: a.sin() * s,
                life: 0.35 + self.random() * 0.35,
                max_life: 0.7,
                size: 1.5 + self.random() * 3.0,
            });
        }
    }

    fn damage(&mut self, x: f64, y: f64) {
        self.lives -= 1;
        self.flash = 0.22;
        self.shake = 0.25;
        self.burst(x, y, 24, 220.0);
        if self.lives <= 0 {
            self.over = true;
            if self.score > self.best { self.best = self.score; save_best(self.best); }
        }
    }

    fn update(&mut self, dt: f64) {
        if self.paused || self.over { return; }

        self.elapsed += dt;
        self.fire_timer -= dt;
        self.flash -= dt;
        self.shake -= dt;

        self.level = 1 + (self.score / 800);
        self.spawn_timer -= dt;

        let spawn_rate = (0.78 - self.level as f64 * 0.035).max(0.22);
        if self.spawn_timer <= 0.0 {
            self.spawn_enemy();
            self.spawn_timer = spawn_rate;
        }

        let mut dx = 0.0;
        let mut dy = 0.0;
        if self.keys[0] { dx -= 1.0; }
        if self.keys[1] { dx += 1.0; }
        if self.keys[2] { dy -= 1.0; }
        if self.keys[3] { dy += 1.0; }

        if let (Some(px), Some(py)) = (self.pointer_x, self.pointer_y) {
            let ax = px - self.player.x;
            let ay = py - self.player.y;
            let dist = (ax * ax + ay * ay).sqrt();
            if dist > 8.0 {
                dx = ax / dist;
                dy = ay / dist;
            }
        }

        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            self.player.x += dx / len * self.player.speed * dt;
            self.player.y += dy / len * self.player.speed * dt;
        }

        self.player.x = self.player.x.clamp(24.0, self.width() - 24.0);
        self.player.y = self.player.y.clamp(70.0, self.height() - 30.0);

        if self.keys[4] { self.shoot(); }

        for b in &mut self.bullets { b.y += b.vy * dt; }
        self.bullets.retain(|b| b.y > -30.0);

        for e in &mut self.enemies {
            e.y += e.vy * dt;
            if e.kind == 1 { e.x += (e.phase + self.elapsed * 4.0).sin() * 42.0 * dt; }
            if e.kind == 2 { e.x += (e.phase + self.elapsed * 2.0).sin() * 18.0 * dt; }
        }

        // Bullet/enemy collisions.
        let mut remove_b = vec![];
        let mut remove_e = vec![];
        for bi in 0..self.bullets.len() {
            for ei in 0..self.enemies.len() {
                if remove_b.contains(&bi) || remove_e.contains(&ei) { continue; }
                let b = &self.bullets[bi];
                let e = &self.enemies[ei];
                let d = ((b.x-e.x).powi(2) + (b.y-e.y).powi(2)).sqrt();
                if d < b.radius + e.size {
                    remove_b.push(bi);
                    self.enemies[ei].hp -= 1;
                    self.burst(e.x, e.y, 5, 95.0);
                    if self.enemies[ei].hp <= 0 {
                        let points = match e.kind { 2 => 260, 1 => 170, _ => 100 };
                        self.score += points;
                        remove_e.push(ei);
                        self.burst(e.x, e.y, 18, 190.0);
                    }
                }
            }
        }
        remove_b.sort_unstable();
        remove_b.dedup();
        remove_e.sort_unstable();
        remove_e.dedup();
        for i in remove_b.into_iter().rev() { self.bullets.remove(i); }
        for i in remove_e.into_iter().rev() { self.enemies.remove(i); }

        // Enemy/player collisions and escaped enemies.
        let mut dead = vec![];
        let mut hit_player = vec![];
        for (i, e) in self.enemies.iter().enumerate() {
            let d = ((self.player.x-e.x).powi(2) + (self.player.y-e.y).powi(2)).sqrt();
            if d < self.player.radius + e.size {
                hit_player.push(i);
            } else if e.y > self.height() + 40.0 {
                dead.push(i);
            }
        }
        hit_player.sort_unstable();
        hit_player.dedup();
        for i in hit_player.iter().rev() {
            let e = self.enemies.remove(*i);
            self.damage(e.x, e.y);
        }
        for i in dead.into_iter().rev() {
            if i < self.enemies.len() { self.enemies.remove(i); }
            self.damage(self.width()/2.0, self.height()-40.0);
            if self.over { break; }
        }

        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vx *= 0.985;
            p.vy *= 0.985;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn draw(&mut self) {
        let w = self.width();
        let h = self.height();
        self.ctx.set_fill_style_str("#050816");
        self.ctx.fill_rect(0.0, 0.0, w, h);

        // Background grid.
        self.ctx.set_stroke_style_str("#10254b");
        self.ctx.set_line_width(1.0);
        let offset = (self.elapsed * 35.0) % 40.0;
        let mut y = offset;
        while y < h {
            self.ctx.begin_path();
            self.ctx.move_to(0.0, y);
            self.ctx.line_to(w, y);
            self.ctx.stroke();
            y += 40.0;
        }
        let mut x = 0.0;
        while x < w {
            self.ctx.begin_path();
            self.ctx.move_to(x, 0.0);
            self.ctx.line_to(x, h);
            self.ctx.stroke();
            x += 40.0;
        }

        let sx = if self.shake > 0.0 { (self.random()-0.5)*10.0 } else { 0.0 };
        let sy = if self.shake > 0.0 { (self.random()-0.5)*10.0 } else { 0.0 };
        self.ctx.save();
        let _ = self.ctx.translate(sx, sy);

        // Particles.
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            self.ctx.set_global_alpha(alpha);
            self.ctx.set_fill_style_str("#55f5ff");
            self.ctx.begin_path();
            let _ = self.ctx.arc(p.x, p.y, p.size, 0.0, PI*2.0);
            self.ctx.fill();
        }
        self.ctx.set_global_alpha(1.0);

        // Bullets.
        self.ctx.set_fill_style_str("#f9ff75");
        for b in &self.bullets {
            self.ctx.begin_path();
            self.ctx.arc(b.x, b.y, b.radius, 0.0, PI*2.0);
            self.ctx.fill();
        }

        // Enemies.
        for e in &self.enemies {
            let fill = match e.kind { 2 => "#ff3d81", 1 => "#a66cff", _ => "#ff9f43" };
            self.ctx.set_fill_style_str(fill);
            self.ctx.begin_path();
            self.ctx.arc(e.x, e.y, e.size, 0.0, PI*2.0);
            self.ctx.fill();
            self.ctx.set_stroke_style_str("#ffffff");
            self.ctx.set_line_width(1.5);
            self.ctx.stroke();
            if e.kind == 2 {
                self.ctx.set_fill_style_str("#ffffff");
                self.ctx.fill_rect(e.x-4.0, e.y-4.0, 8.0, 8.0);
            }
        }

        // Player ship.
        self.ctx.set_fill_style_str("#4df3ff");
        self.ctx.begin_path();
        self.ctx.move_to(self.player.x, self.player.y-25.0);
        self.ctx.line_to(self.player.x-21.0, self.player.y+19.0);
        self.ctx.line_to(self.player.x, self.player.y+10.0);
        self.ctx.line_to(self.player.x+21.0, self.player.y+19.0);
        self.ctx.close_path();
        self.ctx.fill();
        self.ctx.set_stroke_style_str("#ffffff");
        self.ctx.stroke();

        self.ctx.set_fill_style_str("#ffffff");
        self.ctx.begin_path();
        let _ = self.ctx.arc(self.player.x, self.player.y-5.0, 5.0, 0.0, PI*2.0);
        self.ctx.fill();
        self.ctx.restore();

        // HUD.
        self.ctx.set_fill_style_str("#eaf6ff");
        self.ctx.set_font("700 18px system-ui, sans-serif");
        let _ = self.ctx.fill_text(&format!("SCORE  {}", self.score), 18.0, 30.0);
        let _ = self.ctx.fill_text(&format!("BEST  {}", self.best), 18.0, 54.0);
        let _ = self.ctx.fill_text(&format!("LEVEL {}", self.level), w-110.0, 30.0);
        let hearts = "♥".repeat(self.lives.max(0) as usize);
        self.ctx.set_fill_style_str("#ff557f");
        let _ = self.ctx.fill_text(&hearts, w-110.0, 54.0);

        if self.paused || self.over {
            self.ctx.set_fill_style_str("rgba(0,0,0,0.65)");
            self.ctx.fill_rect(0.0, 0.0, w, h);
            self.ctx.set_text_align("center");
            self.ctx.set_fill_style_str("#ffffff");
            self.ctx.set_font("800 42px system-ui, sans-serif");
            let title = if self.over { "SYSTEM DOWN" } else { "PAUSED" };
            let _ = self.ctx.fill_text(title, w/2.0, h/2.0-20.0);
            self.ctx.set_font("600 18px system-ui, sans-serif");
            let sub = if self.over { "Press R or tap RESTART" } else { "Press P or tap RESUME" };
            let _ = self.ctx.fill_text(sub, w/2.0, h/2.0+25.0);
            self.ctx.set_text_align("left");
        }

        if self.flash > 0.0 {
            self.ctx.set_fill_style_str("rgba(255,70,100,0.16)");
            self.ctx.fill_rect(0.0, 0.0, w, h);
        }
    }
}

fn save_best(best: u32) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("cyber_defender_best", &best.to_string());
        }
    }
}

fn load_best() -> u32 {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("cyber_defender_best").ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn canvas_pos(canvas: &HtmlCanvasElement, event: &MouseEvent) -> (f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    (
        (event.client_x() as f64 - rect.left()) * canvas.width() as f64 / rect.width(),
        (event.client_y() as f64 - rect.top()) * canvas.height() as f64 / rect.height(),
    )
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window: Window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("game")
        .ok_or("Canvas not found")?
        .dyn_into()?;

    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or("2D context unavailable")?
        .dyn_into()?;

    let game = Rc::new(RefCell::new(Game {
        canvas: canvas.clone(),
        ctx,
        player: Player { x: 0.0, y: 0.0, radius: 18.0, speed: 390.0, cooldown: 0.0 },
        bullets: vec![],
        enemies: vec![],
        particles: vec![],
        keys: [false; 5],
        pointer_x: None,
        pointer_y: None,
        score: 0,
        best: load_best(),
        lives: 3,
        level: 1,
        spawn_timer: 0.5,
        elapsed: 0.0,
        fire_timer: 0.0,
        paused: false,
        over: false,
        flash: 0.0,
        shake: 0.0,
        seed: js_sys::Date::now() as u32,
    }));
    game.borrow_mut().reset();

    // Keyboard controls.
    {
        let g = game.clone();
        let keydown = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e| {
            let mut game = g.borrow_mut();
            match e.key().to_lowercase().as_str() {
                "arrowleft" | "a" => game.keys[0] = true,
                "arrowright" | "d" => game.keys[1] = true,
                "arrowup" | "w" => game.keys[2] = true,
                "arrowdown" | "s" => game.keys[3] = true,
                " " => game.keys[4] = true,
                "p" | "escape" => game.paused = !game.paused,
                "r" => if game.over { game.reset(); },
                _ => {}
            }
            if ["arrowleft","arrowright","arrowup","arrowdown"," ","a","d","w","s"].contains(&e.key().to_lowercase().as_str()) {
                e.prevent_default();
            }
        });
        window.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
        keydown.forget();
    }
    {
        let g = game.clone();
        let keyup = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e| {
            let mut game = g.borrow_mut();
            match e.key().to_lowercase().as_str() {
                "arrowleft" | "a" => game.keys[0] = false,
                "arrowright" | "d" => game.keys[1] = false,
                "arrowup" | "w" => game.keys[2] = false,
                "arrowdown" | "s" => game.keys[3] = false,
                " " => game.keys[4] = false,
                _ => {}
            }
        });
        window.add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())?;
        keyup.forget();
    }

    // Mouse controls: move ship toward pointer and fire while pressed.
    {
        let g = game.clone();
        let move_mouse = Closure::<dyn FnMut(MouseEvent)>::new(move |e| {
            let (x,y) = canvas_pos(&g.borrow().canvas, &e);
            let mut game = g.borrow_mut();
            game.pointer_x = Some(x);
            game.pointer_y = Some(y);
        });
        canvas.add_event_listener_with_callback("mousemove", move_mouse.as_ref().unchecked_ref())?;
        move_mouse.forget();
    }
    {
        let g = game.clone();
        let down = Closure::<dyn FnMut(MouseEvent)>::new(move |e| {
            let (x,y) = canvas_pos(&g.borrow().canvas, &e);
            let mut game = g.borrow_mut();
            game.pointer_x = Some(x);
            game.pointer_y = Some(y);
            game.keys[4] = true;
        });
        canvas.add_event_listener_with_callback("mousedown", down.as_ref().unchecked_ref())?;
        down.forget();
    }
    {
        let g = game.clone();
        let up = Closure::<dyn FnMut(MouseEvent)>::new(move |_e| {
            g.borrow_mut().keys[4] = false;
        });
        window.add_event_listener_with_callback("mouseup", up.as_ref().unchecked_ref())?;
        up.forget();
    }

    // Touch controls.
    {
        let g = game.clone();
        let touch = Closure::<dyn FnMut(TouchEvent)>::new(move |e| {
            if let Some(t) = e.touches().item(0) {
                let rect = g.borrow().canvas.get_bounding_client_rect();
                let x = (t.client_x() as f64 - rect.left()) * g.borrow().canvas.width() as f64 / rect.width();
                let y = (t.client_y() as f64 - rect.top()) * g.borrow().canvas.height() as f64 / rect.height();
                let mut game = g.borrow_mut();
                game.pointer_x = Some(x);
                game.pointer_y = Some(y);
                game.keys[4] = true;
            }
            e.prevent_default();
        });
        canvas.add_event_listener_with_callback("touchstart", touch.as_ref().unchecked_ref())?;
        canvas.add_event_listener_with_callback("touchmove", touch.as_ref().unchecked_ref())?;
        touch.forget();
    }
    {
        let g = game.clone();
        let touch_end = Closure::<dyn FnMut(TouchEvent)>::new(move |_e| {
            g.borrow_mut().keys[4] = false;
        });
        canvas.add_event_listener_with_callback("touchend", touch_end.as_ref().unchecked_ref())?;
        touch_end.forget();
    }

    // Game loop.
    let last = Rc::new(RefCell::new(js_sys::Date::now()));
    let raf_holder: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let raf_loop = raf_holder.clone();
    let g = game.clone();
    let l = last.clone();
    *raf_loop.borrow_mut() = Some(Closure::wrap(Box::new(move |_now: f64| {
        let current = js_sys::Date::now();
        let mut dt = (current - *l.borrow()) / 1000.0;
        dt = dt.min(0.05);
        *l.borrow_mut() = current;
        {
            let mut game = g.borrow_mut();
            game.update(dt);
            game.draw();
        }
        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(
                raf_holder.borrow().as_ref().unwrap().as_ref().unchecked_ref()
            );
        }
    }) as Box<dyn FnMut(f64)>));

    window.request_animation_frame(
        raf_loop.borrow().as_ref().unwrap().as_ref().unchecked_ref()
    )?;

    Ok(())
}
