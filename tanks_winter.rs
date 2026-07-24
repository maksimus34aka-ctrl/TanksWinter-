// tanks_winter.rs — Танки (зима) на Rust

use std::io::{self, Write, stdout};
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;
use termion::{clear, cursor, color, style};
use termion::input::TermRead;

const W: usize = 30;
const H: usize = 20;
const EMPTY: char = ' ';
const WALL: char = '#';
const SNOW: char = '.';
const ICE: char = '~';
const TANK: char = '@';
const ENEMY: char = 'E';
const BULLET: char = '*';
const BONUS: char = 'B';

#[derive(Clone)]
struct Tank {
    x: usize,
    y: usize,
    dir: char,
}

struct Bullet {
    x: usize,
    y: usize,
    dx: i32,
    dy: i32,
    player: bool,
}

struct Bonus {
    x: usize,
    y: usize,
    typ: char,
}

struct Game {
    field: Vec<Vec<char>>,
    player: Tank,
    enemies: Vec<Tank>,
    bullets: Vec<Bullet>,
    bonuses: Vec<Bonus>,
    score: i32,
    lives: i32,
    level: i32,
    high_score: i32,
    game_over: bool,
    paused: bool,
}

impl Game {
    fn new() -> Self {
        let mut game = Game {
            field: vec![vec![EMPTY; W]; H],
            player: Tank { x: W/2, y: H/2, dir: 'u' },
            enemies: Vec::new(),
            bullets: Vec::new(),
            bonuses: Vec::new(),
            score: 0,
            lives: 3,
            level: 1,
            high_score: 0,
            game_over: false,
            paused: false,
        };
        game.load_high_score();
        game.init_field();
        game.spawn_enemy();
        game
    }

    fn load_high_score(&mut self) {
        self.high_score = 0;
    }

    fn init_field(&mut self) {
        for y in 0..H {
            for x in 0..W {
                self.field[y][x] = EMPTY;
            }
        }
        for x in 0..W {
            self.field[0][x] = WALL;
            self.field[H-1][x] = WALL;
        }
        for y in 0..H {
            self.field[y][0] = WALL;
            self.field[y][W-1] = WALL;
        }
        let mut rng = rand::thread_rng();
        for y in 2..H-2 {
            for x in 2..W-2 {
                let r: f64 = rng.gen();
                if r < 0.3 {
                    self.field[y][x] = SNOW;
                } else if r < 0.4 {
                    self.field[y][x] = ICE;
                } else if r < 0.48 {
                    self.field[y][x] = WALL;
                }
            }
        }
    }

    fn is_occupied(&self, x: usize, y: usize) -> bool {
        if self.player.x == x && self.player.y == y { return true; }
        for e in &self.enemies { if e.x == x && e.y == y { return true; } }
        for b in &self.bullets { if b.x == x && b.y == y { return true; } }
        false
    }

    fn move_tank(&mut self, tank: &mut Tank, dx: i32, dy: i32) -> bool {
        let nx = tank.x as i32 + dx;
        let ny = tank.y as i32 + dy;
        if nx < 1 || nx >= W as i32 -1 || ny < 1 || ny >= H as i32 -1 { return false; }
        let nxu = nx as usize; let nyu = ny as usize;
        if self.field[nyu][nxu] == WALL { return false; }
        if self.is_occupied(nxu, nyu) { return false; }
        tank.x = nxu; tank.y = nyu;
        true
    }

    fn shoot_player(&mut self) {
        let (dx, dy) = match self.player.dir {
            'u' => (0, -1),
            'd' => (0, 1),
            'l' => (-1, 0),
            'r' => (1, 0),
            _ => (0,0),
        };
        let bx = self.player.x as i32 + dx;
        let by = self.player.y as i32 + dy;
        if bx < 1 || bx >= W as i32 -1 || by < 1 || by >= H as i32 -1 { return; }
        let bxu = bx as usize; let byu = by as usize;
        if self.field[byu][bxu] == WALL {
            self.field[byu][bxu] = EMPTY;
            return;
        }
        self.bullets.push(Bullet { x: bxu, y: byu, dx, dy, player: true });
    }

    fn spawn_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let side = rng.gen_range(0..4);
        let (x, y) = match side {
            0 => (rng.gen_range(2..W-2), 1),
            1 => (rng.gen_range(2..W-2), H-2),
            2 => (1, rng.gen_range(2..H-2)),
            _ => (W-2, rng.gen_range(2..H-2)),
        };
        if self.field[y][x] != EMPTY || self.is_occupied(x, y) { return; }
        self.enemies.push(Tank { x, y, dir: 'u' });
    }

    fn update(&mut self) {
        if self.game_over || self.paused { return; }
        let mut rng = rand::thread_rng();
        // Враги
        for i in 0..self.enemies.len() {
            if rng.gen_range(0..10) < 1 {
                let dirs = ['u','d','l','r'];
                let d = dirs[rng.gen_range(0..4)];
                let (dx, dy) = match d {
                    'u' => (0, -1),
                    'd' => (0, 1),
                    'l' => (-1, 0),
                    'r' => (1, 0),
                    _ => (0,0),
                };
                self.move_tank(&mut self.enemies[i], dx, dy);
            }
            if rng.gen_range(0..50) == 0 {
                // стрельба врага
                let mut dx = 0; let mut dy = 0;
                if self.enemies[i].x < self.player.x { dx = 1; }
                else if self.enemies[i].x > self.player.x { dx = -1; }
                if self.enemies[i].y < self.player.y { dy = 1; }
                else if self.enemies[i].y > self.player.y { dy = -1; }
                if dx != 0 || dy != 0 {
                    let bx = self.enemies[i].x as i32 + dx;
                    let by = self.enemies[i].y as i32 + dy;
                    if bx > 0 && bx < W as i32 -1 && by > 0 && by < H as i32 -1 {
                        let bxu = bx as usize; let byu = by as usize;
                        if self.field[byu][bxu] != WALL {
                            self.bullets.push(Bullet { x: bxu, y: byu, dx, dy, player: false });
                        }
                    }
                }
            }
        }
        // Пули
        let mut i = 0;
        while i < self.bullets.len() {
            let b = &mut self.bullets[i];
            b.x = (b.x as i32 + b.dx) as usize;
            b.y = (b.y as i32 + b.dy) as usize;
            if b.x < 1 || b.x >= W-1 || b.y < 1 || b.y >= H-1 {
                self.bullets.remove(i);
                continue;
            }
            if self.field[b.y][b.x] == WALL {
                self.field[b.y][b.x] = EMPTY;
                self.bullets.remove(i);
                continue;
            }
            let mut hit = false;
            if !b.player {
                if b.x == self.player.x && b.y == self.player.y {
                    self.lives -= 1;
                    hit = true;
                    if self.lives <= 0 {
                        self.game_over = true;
                        if self.score > self.high_score { self.high_score = self.score; }
                    }
                }
            } else {
                let mut j = 0;
                while j < self.enemies.len() {
                    if b.x == self.enemies[j].x && b.y == self.enemies[j].y {
                        self.score += 1;
                        hit = true;
                        self.enemies.remove(j);
                        if self.score % 5 == 0 {
                            self.level += 1;
                            self.bonuses.push(Bonus { x: b.x, y: b.y, typ: 'l' });
                        }
                        break;
                    }
                    j += 1;
                }
            }
            if hit {
                self.bullets.remove(i);
            } else {
                i += 1;
            }
        }
        // Бонусы
        let mut i = 0;
        while i < self.bonuses.len() {
            if self.bonuses[i].x == self.player.x && self.bonuses[i].y == self.player.y {
                if self.bonuses[i].typ == 'l' { self.lives += 1; }
                self.bonuses.remove(i);
            } else {
                i += 1;
            }
        }
        // Спавн врагов
        if self.enemies.len() < (3 + self.level as usize).min(8) {
            if rng.gen_range(0..100) < self.level * 1 {
                self.spawn_enemy();
            }
        }
    }

    fn draw(&self) {
        print!("{}{}", clear::All, cursor::Goto(1,1));
        println!("🎮 TanksWinter  |  Счёт: {}  |  Жизни: {}  |  Уровень: {}  |  Рекорд: {}", self.score, self.lives, self.level, self.high_score);
        if self.paused { println!("⏸ ПАУЗА"); }
        print!("+");
        for _ in 0..W { print!("-"); }
        println!("+");
        for y in 0..H {
            print!("|");
            for x in 0..W {
                let mut ch = self.field[y][x];
                if x == self.player.x && y == self.player.y {
                    ch = TANK;
                } else {
                    let mut found = false;
                    for e in &self.enemies {
                        if e.x == x && e.y == y { ch = ENEMY; found = true; break; }
                    }
                    if !found {
                        for b in &self.bullets {
                            if b.x == x && b.y == y { ch = BULLET; found = true; break; }
                        }
                    }
                    if !found {
                        for bon in &self.bonuses {
                            if bon.x == x && bon.y == y { ch = BONUS; break; }
                        }
                    }
                }
                print!("{}", ch);
            }
            println!("|");
        }
        print!("+");
        for _ in 0..W { print!("-"); }
        println!("+");
        println!("Управление: WASD - движение, Пробел - стрельба, P - пауза, Q - выход");
        stdout().flush().unwrap();
    }

    fn run(&mut self) {
        let stdin = io::stdin();
        let mut keys = stdin.keys();
        let mut last_update = Instant::now();
        while !self.game_over {
            self.draw();
            if let Some(Ok(key)) = keys.next() {
                match key {
                    termion::event::Key::Char('q') => break,
                    termion::event::Key::Char('p') => self.paused = !self.paused,
                    termion::event::Key::Char('w') => { self.player.dir = 'u'; self.move_tank(&mut self.player, 0, -1); }
                    termion::event::Key::Char('s') => { self.player.dir = 'd'; self.move_tank(&mut self.player, 0, 1); }
                    termion::event::Key::Char('a') => { self.player.dir = 'l'; self.move_tank(&mut self.player, -1, 0); }
                    termion::event::Key::Char('d') => { self.player.dir = 'r'; self.move_tank(&mut self.player, 1, 0); }
                    termion::event::Key::Char(' ') => self.shoot_player(),
                    _ => {}
                }
            }
            if last_update.elapsed().as_secs_f64() > 0.1 {
                self.update();
                last_update = Instant::now();
            }
            thread::sleep(Duration::from_millis(20));
        }
        println!("ИГРА ОКОНЧЕНА! Счёт: {}", self.score);
    }
}

fn main() {
    let mut game = Game::new();
    game.run();
}
