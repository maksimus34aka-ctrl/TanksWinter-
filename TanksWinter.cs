// TanksWinter.cs — Танки (зима) на C#

using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

class TanksWinter
{
    const int W = 30, H = 20;
    const char EMPTY = ' ', WALL = '#', SNOW = '.', ICE = '~';
    const char TANK = '@', ENEMY = 'E', BULLET = '*', BONUS = 'B';

    static char[,] field;
    static int playerX, playerY;
    static char playerDir = 'u';
    static List<int[]> enemies = new List<int[]>();
    static List<int[]> bullets = new List<int[]>();
    static List<int[]> bonuses = new List<int[]>();
    static int score = 0, lives = 3, level = 1, highScore = 0;
    static bool gameOver = false, paused = false;
    static Random rand = new Random();
    static DateTime lastUpdate = DateTime.Now;
    static bool quit = false;

    static void LoadHighScore() { highScore = 0; }
    static void SaveHighScore() {}

    static void InitField()
    {
        field = new char[H, W];
        for (int y=0; y<H; y++) for (int x=0; x<W; x++) field[y,x] = EMPTY;
        for (int x=0; x<W; x++) { field[0,x] = WALL; field[H-1,x] = WALL; }
        for (int y=0; y<H; y++) { field[y,0] = WALL; field[y,W-1] = WALL; }
        for (int y=2; y<H-2; y++) for (int x=2; x<W-2; x++) {
            double r = rand.NextDouble();
            if (r<0.3) field[y,x] = SNOW;
            else if (r<0.4) field[y,x] = ICE;
            else if (r<0.48) field[y,x] = WALL;
        }
    }

    static bool IsOccupied(int x, int y)
    {
        if (playerX==x && playerY==y) return true;
        foreach (var e in enemies) if (e[0]==x && e[1]==y) return true;
        foreach (var b in bullets) if (b[0]==x && b[1]==y) return true;
        return false;
    }

    static void MovePlayer(int dx, int dy)
    {
        int nx = playerX+dx, ny = playerY+dy;
        if (nx<1 || nx>=W-1 || ny<1 || ny>=H-1) return;
        if (field[ny,nx] == WALL) return;
        if (IsOccupied(nx, ny)) return;
        playerX = nx; playerY = ny;
    }

    static void ShootPlayer()
    {
        int dx=0, dy=0;
        if (playerDir=='u') dy=-1; else if (playerDir=='d') dy=1; else if (playerDir=='l') dx=-1; else dx=1;
        int bx = playerX+dx, by = playerY+dy;
        if (bx<1 || bx>=W-1 || by<1 || by>=H-1) return;
        if (field[by,bx] == WALL) { field[by,bx] = EMPTY; return; }
        bullets.Add(new int[]{bx, by, dx, dy, 1});
    }

    static void SpawnEnemy()
    {
        int side = rand.Next(4);
        int x=0, y=0;
        if (side==0) { x=rand.Next(W-4)+2; y=1; }
        else if (side==1) { x=rand.Next(W-4)+2; y=H-2; }
        else if (side==2) { x=1; y=rand.Next(H-4)+2; }
        else { x=W-2; y=rand.Next(H-4)+2; }
        if (field[y,x]!=EMPTY || IsOccupied(x,y)) return;
        enemies.Add(new int[]{x, y, 0, 0});
    }

    static void Update()
    {
        if (gameOver || paused) return;
        // Враги
        for (int i=0; i<enemies.Count; i++) {
            var e = enemies[i];
            if (rand.Next(10)<1) {
                int d = rand.Next(4);
                int dx=0, dy=0;
                if (d==0) dy=-1; else if (d==1) dy=1; else if (d==2) dx=-1; else dx=1;
                int nx=e[0]+dx, ny=e[1]+dy;
                if (nx>0 && nx<W-1 && ny>0 && ny<H-1 && field[ny,nx]!=WALL && !IsOccupied(nx, ny)) {
                    e[0]=nx; e[1]=ny;
                }
            }
            if (rand.Next(50)==0) {
                int dx=0, dy=0;
                if (e[0] < playerX) dx=1; else if (e[0] > playerX) dx=-1;
                if (e[1] < playerY) dy=1; else if (e[1] > playerY) dy=-1;
                if (dx!=0 || dy!=0) {
                    int bx=e[0]+dx, by=e[1]+dy;
                    if (bx>0 && bx<W-1 && by>0 && by<H-1 && field[by,bx]!=WALL)
                        bullets.Add(new int[]{bx, by, dx, dy, 0});
                }
            }
        }
        // Пули
        for (int i=0; i<bullets.Count; i++) {
            var b = bullets[i];
            b[0] += b[2]; b[1] += b[3];
            if (b[0]<1 || b[0]>=W-1 || b[1]<1 || b[1]>=H-1) { bullets.RemoveAt(i); i--; continue; }
            if (field[b[1],b[0]] == WALL) { field[b[1],b[0]] = EMPTY; bullets.RemoveAt(i); i--; continue; }
            bool hit = false;
            if (b[4]==0) {
                if (b[0]==playerX && b[1]==playerY) {
                    lives--; hit = true;
                    if (lives<=0) { gameOver=true; if(score>highScore) highScore=score; }
                }
            } else {
                for (int j=0; j<enemies.Count; j++) {
                    var e = enemies[j];
                    if (b[0]==e[0] && b[1]==e[1]) {
                        score++; hit = true;
                        enemies.RemoveAt(j);
                        if (score%5==0) { level++; bonuses.Add(new int[]{e[0], e[1], 'l'}); }
                        break;
                    }
                }
            }
            if (hit) { bullets.RemoveAt(i); i--; }
        }
        // Бонусы
        for (int i=0; i<bonuses.Count; i++) {
            var bon = bonuses[i];
            if (bon[0]==playerX && bon[1]==playerY) {
                if (bon[2]=='l') lives++;
                bonuses.RemoveAt(i); i--;
            }
        }
        // Спавн
        if (enemies.Count < Math.Min(3+level, 8)) {
            if (rand.Next(100) < level*1) SpawnEnemy();
        }
    }

    static void Draw()
    {
        Console.Clear();
        Console.WriteLine($"🎮 TanksWinter  |  Счёт: {score}  |  Жизни: {lives}  |  Уровень: {level}  |  Рекорд: {highScore}");
        if (paused) Console.WriteLine("⏸ ПАУЗА");
        Console.Write("+" + new string('-', W) + "+\n");
        for (int y=0; y<H; y++) {
            Console.Write("|");
            for (int x=0; x<W; x++) {
                char ch = field[y,x];
                if (x==playerX && y==playerY) ch = TANK;
                else {
                    bool found=false;
                    foreach (var e in enemies) if (e[0]==x && e[1]==y) { ch=ENEMY; found=true; break; }
                    if (!found) foreach (var b in bullets) if (b[0]==x && b[1]==y) { ch=BULLET; found=true; break; }
                    if (!found) foreach (var bon in bonuses) if (bon[0]==x && bon[1]==y) { ch=BONUS; found=true; break; }
                }
                Console.Write(ch);
            }
            Console.WriteLine("|");
        }
        Console.Write("+" + new string('-', W) + "+\n");
        Console.WriteLine("Управление: WASD - движение, Пробел - стрельба, P - пауза, Q - выход");
    }

    static int GetInput()
    {
        if (Console.KeyAvailable) {
            var key = Console.ReadKey(true).Key;
            if (key == ConsoleKey.Q) return -1;
            if (key == ConsoleKey.P) return -2;
            if (key == ConsoleKey.W) return 1;
            if (key == ConsoleKey.S) return 2;
            if (key == ConsoleKey.A) return 3;
            if (key == ConsoleKey.D) return 4;
            if (key == ConsoleKey.Spacebar) return 5;
        }
        return 0;
    }

    public static async Task Main()
    {
        LoadHighScore();
        InitField();
        playerX = W/2; playerY = H/2;
        SpawnEnemy();
        lastUpdate = DateTime.Now;
        while (!quit && !gameOver) {
            Draw();
            int inp = GetInput();
            if (inp == -1) { quit=true; break; }
            if (inp == -2) { paused = !paused; continue; }
            if (!paused) {
                if (inp == 1) { playerDir='u'; MovePlayer(0,-1); }
                else if (inp == 2) { playerDir='d'; MovePlayer(0,1); }
                else if (inp == 3) { playerDir='l'; MovePlayer(-1,0); }
                else if (inp == 4) { playerDir='r'; MovePlayer(1,0); }
                else if (inp == 5) ShootPlayer();
                if ((DateTime.Now - lastUpdate).TotalSeconds > 0.1) {
                    Update();
                    lastUpdate = DateTime.Now;
                }
            }
            await Task.Delay(20);
        }
        Console.WriteLine($"ИГРА ОКОНЧЕНА! Счёт: {score}");
        SaveHighScore();
    }
}
