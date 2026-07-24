// TanksWinter.java — Танки (зима) на Java (Swing)

import javax.swing.*;
import java.awt.*;
import java.awt.event.*;
import java.util.*;
import java.util.List;
import java.util.Random;

public class TanksWinter extends JPanel implements ActionListener, KeyListener {
    private static final int W = 30, H = 20;
    private static final int CELL = 25;
    private static final char EMPTY = ' ', WALL = '#', SNOW = '.', ICE = '~';
    private static final char TANK = '@', ENEMY = 'E', BULLET = '*', BONUS = 'B';

    private char[][] field;
    private int playerX, playerY;
    private char playerDir = 'u';
    private List<int[]> enemies = new ArrayList<>();
    private List<int[]> bullets = new ArrayList<>();
    private List<int[]> bonuses = new ArrayList<>();
    private int score = 0, lives = 3, level = 1, highScore = 0;
    private boolean gameOver = false, paused = false;
    private Timer timer;
    private Random rand = new Random();

    public TanksWinter() {
        setPreferredSize(new Dimension(W*CELL, H*CELL));
        setBackground(new Color(200,230,255));
        setFocusable(true);
        addKeyListener(this);
        initGame();
        timer = new Timer(50, this);
        timer.start();
        loadHighScore();
    }

    private void loadHighScore() { highScore = 0; }
    private void saveHighScore() {}

    private void initGame() {
        field = new char[H][W];
        for (int y=0; y<H; y++) for (int x=0; x<W; x++) field[y][x] = EMPTY;
        for (int x=0; x<W; x++) { field[0][x] = WALL; field[H-1][x] = WALL; }
        for (int y=0; y<H; y++) { field[y][0] = WALL; field[y][W-1] = WALL; }
        for (int y=2; y<H-2; y++) for (int x=2; x<W-2; x++) {
            double r = rand.nextDouble();
            if (r<0.3) field[y][x] = SNOW;
            else if (r<0.4) field[y][x] = ICE;
            else if (r<0.48) field[y][x] = WALL;
        }
        playerX = W/2; playerY = H/2;
        enemies.clear(); bullets.clear(); bonuses.clear();
        score = 0; lives = 3; level = 1; gameOver = false; paused = false;
        spawnEnemy();
    }

    private boolean isOccupied(int x, int y) {
        if (playerX==x && playerY==y) return true;
        for (int[] e : enemies) if (e[0]==x && e[1]==y) return true;
        for (int[] b : bullets) if (b[0]==x && b[1]==y) return true;
        return false;
    }

    private void movePlayer(int dx, int dy) {
        int nx = playerX+dx, ny = playerY+dy;
        if (nx<1 || nx>=W-1 || ny<1 || ny>=H-1) return;
        if (field[ny][nx] == WALL) return;
        if (isOccupied(nx, ny)) return;
        playerX = nx; playerY = ny;
    }

    private void shootPlayer() {
        int dx=0, dy=0;
        if (playerDir=='u') dy=-1; else if (playerDir=='d') dy=1; else if (playerDir=='l') dx=-1; else dx=1;
        int bx = playerX+dx, by = playerY+dy;
        if (bx<1 || bx>=W-1 || by<1 || by>=H-1) return;
        if (field[by][bx] == WALL) { field[by][bx] = EMPTY; return; }
        bullets.add(new int[]{bx, by, dx, dy, 1}); // 1=player
    }

    private void spawnEnemy() {
        int side = rand.nextInt(4);
        int x=0, y=0;
        if (side==0) { x=rand.nextInt(W-4)+2; y=1; }
        else if (side==1) { x=rand.nextInt(W-4)+2; y=H-2; }
        else if (side==2) { x=1; y=rand.nextInt(H-4)+2; }
        else { x=W-2; y=rand.nextInt(H-4)+2; }
        if (field[y][x]!=EMPTY || isOccupied(x,y)) return;
        enemies.add(new int[]{x, y, 0, 0}); // dir, cooldown хранятся отдельно
    }

    @Override
    public void actionPerformed(ActionEvent e) {
        if (gameOver || paused) return;
        // Движение врагов
        for (int i=0; i<enemies.size(); i++) {
            int[] e = enemies.get(i);
            if (rand.nextInt(10)<1) {
                int d = rand.nextInt(4);
                int dx=0, dy=0;
                if (d==0) dy=-1; else if (d==1) dy=1; else if (d==2) dx=-1; else dx=1;
                int nx=e[0]+dx, ny=e[1]+dy;
                if (nx>0 && nx<W-1 && ny>0 && ny<H-1 && field[ny][nx]!=WALL && !isOccupied(nx, ny)) {
                    e[0]=nx; e[1]=ny;
                }
            }
            if (rand.nextInt(50)==0) {
                // враг стреляет
                int dx=0, dy=0;
                // направление к игроку
                if (e[0] < playerX) dx=1; else if (e[0] > playerX) dx=-1;
                if (e[1] < playerY) dy=1; else if (e[1] > playerY) dy=-1;
                if (dx!=0 || dy!=0) {
                    int bx=e[0]+dx, by=e[1]+dy;
                    if (bx>0 && bx<W-1 && by>0 && by<H-1 && field[by][bx]!=WALL) {
                        bullets.add(new int[]{bx, by, dx, dy, 0}); // 0=enemy
                    }
                }
            }
        }
        // Пули
        for (int i=0; i<bullets.size(); i++) {
            int[] b = bullets.get(i);
            b[0] += b[2]; b[1] += b[3];
            if (b[0]<1 || b[0]>=W-1 || b[1]<1 || b[1]>=H-1) { bullets.remove(i); i--; continue; }
            if (field[b[1]][b[0]] == WALL) { field[b[1]][b[0]] = EMPTY; bullets.remove(i); i--; continue; }
            boolean hit = false;
            if (b[4]==0) { // enemy bullet
                if (b[0]==playerX && b[1]==playerY) {
                    lives--; hit = true;
                    if (lives<=0) { gameOver=true; if(score>highScore) highScore=score; }
                }
            } else { // player bullet
                for (int j=0; j<enemies.size(); j++) {
                    int[] e = enemies.get(j);
                    if (b[0]==e[0] && b[1]==e[1]) {
                        score++; hit = true;
                        enemies.remove(j);
                        if (score%5==0) { level++; bonuses.add(new int[]{e[0], e[1], 'l'}); }
                        break;
                    }
                }
            }
            if (hit) { bullets.remove(i); i--; }
        }
        // Бонусы
        for (int i=0; i<bonuses.size(); i++) {
            int[] bon = bonuses.get(i);
            if (bon[0]==playerX && bon[1]==playerY) {
                if (bon[2]=='l') lives++;
                bonuses.remove(i); i--;
            }
        }
        // Спавн врагов
        if (enemies.size() < Math.min(3+level, 8)) {
            if (rand.nextInt(100) < level*1) spawnEnemy();
        }
        repaint();
    }

    @Override
    public void paintComponent(Graphics g) {
        super.paintComponent(g);
        Graphics2D g2 = (Graphics2D) g;
        for (int y=0; y<H; y++) for (int x=0; x<W; x++) {
            char ch = field[y][x];
            Color col = Color.WHITE;
            if (ch == WALL) col = Color.DARK_GRAY;
            else if (ch == SNOW) col = new Color(230, 240, 255);
            else if (ch == ICE) col = new Color(180, 220, 255);
            else col = new Color(200, 230, 255);
            g2.setColor(col);
            g2.fillRect(x*CELL, y*CELL, CELL, CELL);
            if (ch == WALL) {
                g2.setColor(Color.BLACK);
                g2.drawRect(x*CELL, y*CELL, CELL, CELL);
            }
        }
        // Игрок
        g2.setColor(Color.GREEN);
        g2.fillRect(playerX*CELL+2, playerY*CELL+2, CELL-4, CELL-4);
        // Враги
        g2.setColor(Color.RED);
        for (int[] e : enemies) g2.fillRect(e[0]*CELL+2, e[1]*CELL+2, CELL-4, CELL-4);
        // Пули
        g2.setColor(Color.YELLOW);
        for (int[] b : bullets) g2.fillOval(b[0]*CELL+8, b[1]*CELL+8, 10, 10);
        // Бонусы
        g2.setColor(Color.CYAN);
        for (int[] bon : bonuses) g2.drawString("B", bon[0]*CELL+8, bon[1]*CELL+18);
        // Информация
        g2.setColor(Color.BLACK);
        g2.drawString("Счёт: "+score+"  Жизни: "+lives+"  Уровень: "+level+"  Рекорд: "+highScore, 10, 20);
        if (paused) g2.drawString("ПАУЗА", W*CELL/2-30, H*CELL/2);
        if (gameOver) {
            g2.drawString("ИГРА ОКОНЧЕНА! Нажмите R для рестарта", W*CELL/2-80, H*CELL/2+20);
        }
    }

    @Override
    public void keyPressed(KeyEvent e) {
        int key = e.getKeyCode();
        if (gameOver && key == KeyEvent.VK_R) { initGame(); return; }
        if (key == KeyEvent.VK_ESCAPE) System.exit(0);
        if (key == KeyEvent.VK_P) { paused = !paused; return; }
        if (paused) return;
        int dx=0, dy=0;
        if (key == KeyEvent.VK_W) { dy=-1; playerDir='u'; }
        else if (key == KeyEvent.VK_S) { dy=1; playerDir='d'; }
        else if (key == KeyEvent.VK_A) { dx=-1; playerDir='l'; }
        else if (key == KeyEvent.VK_D) { dx=1; playerDir='r'; }
        if (dx!=0 || dy!=0) movePlayer(dx, dy);
        if (key == KeyEvent.VK_SPACE) shootPlayer();
    }
    @Override public void keyReleased(KeyEvent e) {}
    @Override public void keyTyped(KeyEvent e) {}

    public static void main(String[] args) {
        JFrame frame = new JFrame("🎮 TanksWinter");
        TanksWinter game = new TanksWinter();
        frame.add(game);
        frame.pack();
        frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        frame.setVisible(true);
        frame.setLocationRelativeTo(null);
    }
}
