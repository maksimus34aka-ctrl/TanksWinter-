// tanks_winter.js — Танки (зима) на JavaScript (Node.js)

const readline = require('readline');
const keypress = require('keypress');

const W = 30, H = 20;
const EMPTY = ' ', WALL = '#', SNOW = '.', ICE = '~';
const TANK = '@', ENEMY = 'E', BULLET = '*', BONUS = 'B';

let field = [];
let player = {x: Math.floor(W/2), y: Math.floor(H/2), dir: 'u'};
let enemies = [];
let bullets = [];
let bonuses = [];
let score = 0, lives = 3, level = 1, highScore = 0;
let gameOver = false, paused = false;
let lastUpdate = Date.now();
let quit = false;

function loadHighScore() { highScore = 0; }
function saveHighScore() {}

function initField() {
    field = Array.from({length: H}, () => Array(W).fill(EMPTY));
    for (let x=0; x<W; x++) { field[0][x] = WALL; field[H-1][x] = WALL; }
    for (let y=0; y<H; y++) { field[y][0] = WALL; field[y][W-1] = WALL; }
    for (let y=2; y<H-2; y++) {
        for (let x=2; x<W-2; x++) {
            const r = Math.random();
            if (r < 0.3) field[y][x] = SNOW;
            else if (r < 0.4) field[y][x] = ICE;
            else if (r < 0.48) field[y][x] = WALL;
        }
    }
}

function isOccupied(x, y) {
    if (player.x === x && player.y === y) return true;
    for (const e of enemies) if (e.x === x && e.y === y) return true;
    for (const b of bullets) if (b.x === x && b.y === y) return true;
    return false;
}

function moveTank(tank, dx, dy) {
    const nx = tank.x + dx, ny = tank.y + dy;
    if (nx < 1 || nx >= W-1 || ny < 1 || ny >= H-1) return false;
    if (field[ny][nx] === WALL) return false;
    if (isOccupied(nx, ny)) return false;
    tank.x = nx; tank.y = ny;
    return true;
}

function shootPlayer() {
    let dx=0, dy=0;
    if (player.dir === 'u') dy = -1;
    else if (player.dir === 'd') dy = 1;
    else if (player.dir === 'l') dx = -1;
    else dx = 1;
    const bx = player.x + dx, by = player.y + dy;
    if (bx < 1 || bx >= W-1 || by < 1 || by >= H-1) return;
    if (field[by][bx] === WALL) { field[by][bx] = EMPTY; return; }
    bullets.push({x: bx, y: by, dx, dy, player: true});
}

function spawnEnemy() {
    const side = Math.floor(Math.random() * 4);
    let x, y;
    if (side === 0) { x = Math.floor(Math.random()*(W-4))+2; y = 1; }
    else if (side === 1) { x = Math.floor(Math.random()*(W-4))+2; y = H-2; }
    else if (side === 2) { x = 1; y = Math.floor(Math.random()*(H-4))+2; }
    else { x = W-2; y = Math.floor(Math.random()*(H-4))+2; }
    if (field[y][x] !== EMPTY || isOccupied(x,y)) return;
    enemies.push({x, y, dir: 'u'});
}

function update() {
    if (gameOver || paused) return;
    // Враги
    for (let i=0; i<enemies.length; i++) {
        const e = enemies[i];
        if (Math.random() < 0.1) {
            const dirs = ['u','d','l','r'];
            const d = dirs[Math.floor(Math.random()*4)];
            let dx=0, dy=0;
            if (d==='u') dy=-1; else if (d==='d') dy=1; else if (d==='l') dx=-1; else dx=1;
            moveTank(e, dx, dy);
        }
        if (Math.random() < 0.02) {
            let dx=0, dy=0;
            if (e.x < player.x) dx = 1; else if (e.x > player.x) dx = -1;
            if (e.y < player.y) dy = 1; else if (e.y > player.y) dy = -1;
            if (dx!==0 || dy!==0) {
                const bx = e.x + dx, by = e.y + dy;
                if (bx>0 && bx<W-1 && by>0 && by<H-1 && field[by][bx]!==WALL) {
                    bullets.push({x: bx, y: by, dx, dy, player: false});
                }
            }
        }
    }
    // Пули
    for (let i=0; i<bullets.length; i++) {
        const b = bullets[i];
        b.x += b.dx; b.y += b.dy;
        if (b.x<1 || b.x>=W-1 || b.y<1 || b.y>=H-1) { bullets.splice(i,1); i--; continue; }
        if (field[b.y][b.x] === WALL) { field[b.y][b.x] = EMPTY; bullets.splice(i,1); i--; continue; }
        let hit = false;
        if (!b.player) {
            if (b.x === player.x && b.y === player.y) {
                lives--; hit = true;
                if (lives<=0) { gameOver=true; if(score>highScore) highScore=score; }
            }
        } else {
            for (let j=0; j<enemies.length; j++) {
                if (b.x === enemies[j].x && b.y === enemies[j].y) {
                    score++; hit = true;
                    enemies.splice(j,1);
                    if (score%5===0) { level++; bonuses.push({x: b.x, y: b.y, type: 'l'}); }
                    break;
                }
            }
        }
        if (hit) { bullets.splice(i,1); i--; }
    }
    // Бонусы
    for (let i=0; i<bonuses.length; i++) {
        if (bonuses[i].x === player.x && bonuses[i].y === player.y) {
            if (bonuses[i].type === 'l') lives++;
            bonuses.splice(i,1); i--;
        }
    }
    // Спавн
    if (enemies.length < Math.min(3+level, 8)) {
        if (Math.random() < 0.01 * level) spawnEnemy();
    }
}

function draw() {
    console.clear();
    console.log(`🎮 TanksWinter  |  Счёт: ${score}  |  Жизни: ${lives}  |  Уровень: ${level}  |  Рекорд: ${highScore}`);
    if (paused) console.log("⏸ ПАУЗА");
    let top = '+' + '-'.repeat(W) + '+';
    console.log(top);
    for (let y=0; y<H; y++) {
        let line = '|';
        for (let x=0; x<W; x++) {
            let ch = field[y][x];
            if (x===player.x && y===player.y) ch = TANK;
            else {
                let found = false;
                for (const e of enemies) if (e.x===x && e.y===y) { ch=ENEMY; found=true; break; }
                if (!found) for (const b of bullets) if (b.x===x && b.y===y) { ch=BULLET; found=true; break; }
                if (!found) for (const bon of bonuses) if (bon.x===x && bon.y===y) { ch=BONUS; found=true; break; }
            }
            line += ch;
        }
        line += '|';
        console.log(line);
    }
    console.log(top);
    console.log("Управление: WASD - движение, Пробел - стрельба, P - пауза, Q - выход");
}

function setupInput() {
    keypress(process.stdin);
    process.stdin.on('keypress', (ch, key) => {
        if (key && key.ctrl && key.name === 'c') { quit = true; process.exit(); }
        if (key && key.name === 'q') { quit = true; }
        if (key && key.name === 'p') { paused = !paused; return; }
        if (gameOver || paused) return;
        if (key && key.name === 'w') { player.dir='u'; moveTank(player, 0, -1); }
        if (key && key.name === 's') { player.dir='d'; moveTank(player, 0, 1); }
        if (key && key.name === 'a') { player.dir='l'; moveTank(player, -1, 0); }
        if (key && key.name === 'd') { player.dir='r'; moveTank(player, 1, 0); }
        if (key && key.name === 'space') { shootPlayer(); }
    });
    process.stdin.setRawMode(true);
    process.stdin.resume();
}

function gameLoop() {
    if (gameOver) {
        draw();
        console.log(`ИГРА ОКОНЧЕНА! Счёт: ${score}`);
        saveHighScore();
        process.exit(0);
    }
    const now = Date.now();
    if ((now - lastUpdate) > 100) {
        update();
        lastUpdate = now;
    }
    draw();
    setTimeout(gameLoop, 20);
}

loadHighScore();
initField();
setupInput();
player = {x: Math.floor(W/2), y: Math.floor(H/2), dir: 'u'};
spawnEnemy();
lastUpdate = Date.now();
gameLoop();
