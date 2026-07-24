// tanks_winter.cpp — Танки (зима) на C++ (ncurses)

#include <ncurses.h>
#include <cstdlib>
#include <ctime>
#include <vector>
#include <deque>
#include <algorithm>
#include <thread>
#include <chrono>

using namespace std;

const int W = 30, H = 20;
const char EMPTY = ' ', WALL = '#', SNOW = '.', ICE = '~';
const char TANK = '@', ENEMY = 'E', BULLET = '*', BONUS = 'B';

struct Point { int x, y; };
struct Tank {
    int x, y;
    char dir; // 'u','d','l','r'
    bool isPlayer;
    int hp, speed, cooldown;
};
struct Bullet { int x, y, dx, dy; bool player; };
struct Bonus { int x, y; char type; };

class Game {
private:
    vector<vector<char>> field;
    Tank player;
    vector<Tank> enemies;
    vector<Bullet> bullets;
    vector<Bonus> bonuses;
    int score, highScore, lives, level;
    bool gameOver, paused;

public:
    Game() {
        score = 0; lives = 3; level = 1; gameOver = false; paused = false;
        loadHighScore();
        field.assign(H, vector<char>(W, EMPTY));
        initField();
        // Игрок в центре
        player = {W/2, H/2, 'u', true, 3, 1, 0};
        spawnEnemy();
    }

    void loadHighScore() {
        // упрощённо
        highScore = 0;
    }
    void saveHighScore() {}

    void initField() {
        for (int y=0; y<H; y++) for (int x=0; x<W; x++) field[y][x] = EMPTY;
        for (int x=0; x<W; x++) { field[0][x] = WALL; field[H-1][x] = WALL; }
        for (int y=0; y<H; y++) { field[y][0] = WALL; field[y][W-1] = WALL; }
        for (int y=2; y<H-2; y++)
            for (int x=2; x<W-2; x++) {
                float r = rand() / (float)RAND_MAX;
                if (r < 0.3) field[y][x] = SNOW;
                else if (r < 0.4) field[y][x] = ICE;
                else if (r < 0.48) field[y][x] = WALL;
            }
    }

    bool isOccupied(int x, int y) {
        if (player.x == x && player.y == y) return true;
        for (auto& e : enemies) if (e.x==x && e.y==y) return true;
        for (auto& b : bullets) if (b.x==x && b.y==y) return true;
        return false;
    }

    void moveTank(Tank& t, int dx, int dy) {
        int nx = t.x+dx, ny = t.y+dy;
        if (nx<1 || nx>=W-1 || ny<1 || ny>=H-1) return;
        if (field[ny][nx] == WALL) return;
        if (isOccupied(nx, ny)) return;
        t.x = nx; t.y = ny;
    }

    void shoot(Tank& t) {
        if (t.cooldown > 0) return;
        int dx=0, dy=0;
        if (t.dir=='u') dy=-1; else if (t.dir=='d') dy=1; else if (t.dir=='l') dx=-1; else dx=1;
        int bx = t.x+dx, by = t.y+dy;
        if (bx<1 || bx>=W-1 || by<1 || by>=H-1) return;
        if (field[by][bx] == WALL) { field[by][bx] = EMPTY; return; }
        bullets.push_back({bx, by, dx, dy, t.isPlayer});
        t.cooldown = 5;
    }

    void spawnEnemy() {
        int side = rand()%4;
        int x=0, y=0;
        if (side==0) { x=rand()%(W-4)+2; y=1; }
        else if (side==1) { x=rand()%(W-4)+2; y=H-2; }
        else if (side==2) { x=1; y=rand()%(H-4)+2; }
        else { x=W-2; y=rand()%(H-4)+2; }
        if (field[y][x]!=EMPTY || isOccupied(x,y)) return;
        Tank e = {x, y, 'u', false, 1, 1, 0};
        enemies.push_back(e);
    }

    void update() {
        if (gameOver || paused) return;
        // Движение врагов
        for (auto& e : enemies) {
            if (rand()%10 < 1) {
                char d = "udlr"[rand()%4];
                int dx=0, dy=0;
                if (d=='u') dy=-1; else if (d=='d') dy=1; else if (d=='l') dx=-1; else dx=1;
                if (moveTankNoCheck(e, dx, dy)) e.dir = d;
            }
            if (rand()%50 == 0) shoot(e);
        }
        // Пули
        for (auto it = bullets.begin(); it != bullets.end(); ) {
            it->x += it->dx; it->y += it->dy;
            if (it->x<1 || it->x>=W-1 || it->y<1 || it->y>=H-1) { it = bullets.erase(it); continue; }
            if (field[it->y][it->x] == WALL) { field[it->y][it->x] = EMPTY; it = bullets.erase(it); continue; }
            bool hit = false;
            if (!it->player) {
                if (it->x == player.x && it->y == player.y) {
                    lives--; hit = true;
                    if (lives<=0) { gameOver=true; if(score>highScore) highScore=score; }
                }
            } else {
                for (auto et = enemies.begin(); et != enemies.end(); ) {
                    if (it->x == et->x && it->y == et->y) {
                        score++; hit = true;
                        et = enemies.erase(et);
                        if (score%5==0) { level++; bonuses.push_back({et->x, et->y, 'l'}); }
                        break;
                    } else ++et;
                }
            }
            if (hit) { it = bullets.erase(it); }
            else ++it;
        }
        // Бонусы
        for (auto it = bonuses.begin(); it != bonuses.end(); ) {
            if (it->x == player.x && it->y == player.y) {
                if (it->type == 'l') lives++;
                it = bonuses.erase(it);
            } else ++it;
        }
        // Спавн врагов
        if (enemies.size() < min(3+level, 8)) {
            if (rand()%100 < 1*level) spawnEnemy();
        }
        // Кулдауны
        if (player.cooldown>0) player.cooldown--;
        for (auto& e : enemies) if (e.cooldown>0) e.cooldown--;
    }

    bool moveTankNoCheck(Tank& t, int dx, int dy) {
        int nx = t.x+dx, ny = t.y+dy;
        if (nx<1 || nx>=W-1 || ny<1 || ny>=H-1) return false;
        if (field[ny][nx] == WALL) return false;
        if (isOccupied(nx, ny)) return false;
        t.x = nx; t.y = ny;
        return true;
    }

    void draw() {
        clear();
        printw("🎮 TanksWinter  |  Счёт: %d  |  Жизни: %d  |  Уровень: %d  |  Рекорд: %d\n", score, lives, level, highScore);
        if (paused) printw("⏸ ПАУЗА\n");
        printw("+");
        for (int i=0; i<W; i++) printw("-");
        printw("+\n");
        for (int y=0; y<H; y++) {
            printw("|");
            for (int x=0; x<W; x++) {
                char ch = field[y][x];
                if (x==player.x && y==player.y) ch = TANK;
                else {
                    bool found=false;
                    for (auto& e : enemies) if (e.x==x && e.y==y) { ch=ENEMY; found=true; break; }
                    if (!found) for (auto& b : bullets) if (b.x==x && b.y==y) { ch=BULLET; found=true; break; }
                    if (!found) for (auto& b : bonuses) if (b.x==x && b.y==y) { ch=BONUS; found=true; break; }
                }
                printw("%c", ch);
            }
            printw("|\n");
        }
        printw("+");
        for (int i=0; i<W; i++) printw("-");
        printw("+\n");
        printw("Управление: WASD - движение, Пробел - стрельба, P - пауза, Q - выход\n");
        refresh();
    }

    void run() {
        initscr();
        cbreak();
        noecho();
        keypad(stdscr, TRUE);
        nodelay(stdscr, TRUE);
        curs_set(0);
        srand(time(nullptr));

        int ch;
        while (!gameOver) {
            draw();
            ch = getch();
            if (ch == 'q' || ch == 'Q') break;
            if (ch == 'p' || ch == 'P') { paused = !paused; continue; }
            if (!paused) {
                int dx=0, dy=0;
                if (ch == 'w' || ch == 'W') { dy=-1; player.dir='u'; }
                else if (ch == 's' || ch == 'S') { dy=1; player.dir='d'; }
                else if (ch == 'a' || ch == 'A') { dx=-1; player.dir='l'; }
                else if (ch == 'd' || ch == 'D') { dx=1; player.dir='r'; }
                if (dx!=0 || dy!=0) moveTank(player, dx, dy);
                if (ch == ' ') shoot(player);
                update();
            }
            this_thread::sleep_for(chrono::milliseconds(50));
        }
        endwin();
        cout << "ИГРА ОКОНЧЕНА! Счёт: " << score << endl;
        saveHighScore();
    }
};

int main() {
    Game game;
    game.run();
    return 0;
}
