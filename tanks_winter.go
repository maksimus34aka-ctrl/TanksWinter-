// tanks_winter.go — Танки (зима) на Go

package main

import (
	"fmt"
	"math/rand"
	"os"
	"os/exec"
	"time"

	"github.com/eiannone/keyboard"
)

const W = 30
const H = 20
const EMPTY = ' '
const WALL = '#'
const SNOW = '.'
const ICE = '~'
const TANK = '@'
const ENEMY = 'E'
const BULLET = '*'
const BONUS = 'B'

type Tank struct {
	x, y int
	dir  byte // 'u','d','l','r'
}

type Bullet struct {
	x, y, dx, dy int
	player       bool
}

type Bonus struct {
	x, y int
	typ  byte
}

var field [][]byte
var player Tank
var enemies []Tank
var bullets []Bullet
var bonuses []Bonus
var score, lives, level, highScore int
var gameOver, paused bool
var randSrc *rand.Rand

func clear() {
	cmd := exec.Command("clear")
	cmd.Stdout = os.Stdout
	cmd.Run()
}

func loadHighScore() { highScore = 0 }
func saveHighScore() {}

func initField() {
	field = make([][]byte, H)
	for y := 0; y < H; y++ {
		field[y] = make([]byte, W)
		for x := 0; x < W; x++ {
			field[y][x] = EMPTY
		}
	}
	for x := 0; x < W; x++ {
		field[0][x] = WALL
		field[H-1][x] = WALL
	}
	for y := 0; y < H; y++ {
		field[y][0] = WALL
		field[y][W-1] = WALL
	}
	for y := 2; y < H-2; y++ {
		for x := 2; x < W-2; x++ {
			r := randSrc.Float64()
			if r < 0.3 {
				field[y][x] = SNOW
			} else if r < 0.4 {
				field[y][x] = ICE
			} else if r < 0.48 {
				field[y][x] = WALL
			}
		}
	}
}

func isOccupied(x, y int) bool {
	if player.x == x && player.y == y {
		return true
	}
	for _, e := range enemies {
		if e.x == x && e.y == y {
			return true
		}
	}
	for _, b := range bullets {
		if b.x == x && b.y == y {
			return true
		}
	}
	return false
}

func moveTank(t *Tank, dx, dy int) bool {
	nx, ny := t.x+dx, t.y+dy
	if nx < 1 || nx >= W-1 || ny < 1 || ny >= H-1 {
		return false
	}
	if field[ny][nx] == WALL {
		return false
	}
	if isOccupied(nx, ny) {
		return false
	}
	t.x, t.y = nx, ny
	return true
}

func shootPlayer() {
	var dx, dy int
	switch player.dir {
	case 'u':
		dy = -1
	case 'd':
		dy = 1
	case 'l':
		dx = -1
	case 'r':
		dx = 1
	}
	bx, by := player.x+dx, player.y+dy
	if bx < 1 || bx >= W-1 || by < 1 || by >= H-1 {
		return
	}
	if field[by][bx] == WALL {
		field[by][bx] = EMPTY
		return
	}
	bullets = append(bullets, Bullet{bx, by, dx, dy, true})
}

func spawnEnemy() {
	side := randSrc.Intn(4)
	var x, y int
	switch side {
	case 0:
		x = randSrc.Intn(W-4) + 2
		y = 1
	case 1:
		x = randSrc.Intn(W-4) + 2
		y = H - 2
	case 2:
		x = 1
		y = randSrc.Intn(H-4) + 2
	case 3:
		x = W - 2
		y = randSrc.Intn(H-4) + 2
	}
	if field[y][x] != EMPTY || isOccupied(x, y) {
		return
	}
	enemies = append(enemies, Tank{x, y, 'u'})
}

func update() {
	if gameOver || paused {
		return
	}
	// Враги
	for i := 0; i < len(enemies); i++ {
		if randSrc.Intn(10) < 1 {
			dirs := []byte{'u', 'd', 'l', 'r'}
			d := dirs[randSrc.Intn(4)]
			var dx, dy int
			switch d {
			case 'u':
				dy = -1
			case 'd':
				dy = 1
			case 'l':
				dx = -1
			case 'r':
				dx = 1
			}
			moveTank(&enemies[i], dx, dy)
		}
		if randSrc.Intn(50) == 0 {
			// враг стреляет в сторону игрока
			dx, dy := 0, 0
			if enemies[i].x < player.x {
				dx = 1
			} else if enemies[i].x > player.x {
				dx = -1
			}
			if enemies[i].y < player.y {
				dy = 1
			} else if enemies[i].y > player.y {
				dy = -1
			}
			if dx != 0 || dy != 0 {
				bx, by := enemies[i].x+dx, enemies[i].y+dy
				if bx > 0 && bx < W-1 && by > 0 && by < H-1 && field[by][bx] != WALL {
					bullets = append(bullets, Bullet{bx, by, dx, dy, false})
				}
			}
		}
	}
	// Пули
	for i := 0; i < len(bullets); i++ {
		b := &bullets[i]
		b.x += b.dx
		b.y += b.dy
		if b.x < 1 || b.x >= W-1 || b.y < 1 || b.y >= H-1 {
			bullets = append(bullets[:i], bullets[i+1:]...)
			i--
			continue
		}
		if field[b.y][b.x] == WALL {
			field[b.y][b.x] = EMPTY
			bullets = append(bullets[:i], bullets[i+1:]...)
			i--
			continue
		}
		hit := false
		if !b.player {
			if b.x == player.x && b.y == player.y {
				lives--
				hit = true
				if lives <= 0 {
					gameOver = true
					if score > highScore {
						highScore = score
					}
				}
			}
		} else {
			for j := 0; j < len(enemies); j++ {
				if b.x == enemies[j].x && b.y == enemies[j].y {
					score++
					hit = true
					enemies = append(enemies[:j], enemies[j+1:]...)
					if score%5 == 0 {
						level++
						bonuses = append(bonuses, Bonus{enemies[j].x, enemies[j].y, 'l'})
					}
					break
				}
			}
		}
		if hit {
			bullets = append(bullets[:i], bullets[i+1:]...)
			i--
		}
	}
	// Бонусы
	for i := 0; i < len(bonuses); i++ {
		if bonuses[i].x == player.x && bonuses[i].y == player.y {
			if bonuses[i].typ == 'l' {
				lives++
			}
			bonuses = append(bonuses[:i], bonuses[i+1:]...)
			i--
		}
	}
	// Спавн врагов
	if len(enemies) < min(3+level, 8) {
		if randSrc.Intn(100) < level*1 {
			spawnEnemy()
		}
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func draw() {
	clear()
	fmt.Printf("🎮 TanksWinter  |  Счёт: %d  |  Жизни: %d  |  Уровень: %d  |  Рекорд: %d\n", score, lives, level, highScore)
	if paused {
		fmt.Println("⏸ ПАУЗА")
	}
	fmt.Print("+" + strings.Repeat("-", W) + "+\n")
	for y := 0; y < H; y++ {
		fmt.Print("|")
		for x := 0; x < W; x++ {
			ch := field[y][x]
			if x == player.x && y == player.y {
				ch = TANK
			} else {
				found := false
				for _, e := range enemies {
					if e.x == x && e.y == y {
						ch = ENEMY
						found = true
						break
					}
				}
				if !found {
					for _, b := range bullets {
						if b.x == x && b.y == y {
							ch = BULLET
							found = true
							break
						}
					}
				}
				if !found {
					for _, bon := range bonuses {
						if bon.x == x && bon.y == y {
							ch = BONUS
							break
						}
					}
				}
			}
			fmt.Printf("%c", ch)
		}
		fmt.Println("|")
	}
	fmt.Print("+" + strings.Repeat("-", W) + "+\n")
	fmt.Println("Управление: WASD - движение, Пробел - стрельба, P - пауза, Q - выход")
}

func main() {
	randSrc = rand.New(rand.NewSource(time.Now().UnixNano()))
	loadHighScore()
	initField()
	player = Tank{W / 2, H / 2, 'u'}
	lives = 3
	score = 0
	level = 1
	gameOver = false
	paused = false
	spawnEnemy()

	if err := keyboard.Open(); err != nil {
		panic(err)
	}
	defer keyboard.Close()

	lastUpdate := time.Now()
	for !gameOver {
		draw()
		// обработка ввода
		_, key, err := keyboard.GetKey()
		if err != nil {
			continue
		}
		if key == keyboard.KeyEsc {
			break
		}
		if key == 'q' || key == 'Q' {
			break
		}
		if key == 'p' || key == 'P' {
			paused = !paused
			continue
		}
		if !paused {
			switch key {
			case 'w', 'W':
				player.dir = 'u'
				moveTank(&player, 0, -1)
			case 's', 'S':
				player.dir = 'd'
				moveTank(&player, 0, 1)
			case 'a', 'A':
				player.dir = 'l'
				moveTank(&player, -1, 0)
			case 'd', 'D':
				player.dir = 'r'
				moveTank(&player, 1, 0)
			case ' ':
				shootPlayer()
			}
			if time.Since(lastUpdate).Seconds() > 0.1 {
				update()
				lastUpdate = time.Now()
			}
		}
		time.Sleep(20 * time.Millisecond)
	}
	fmt.Printf("ИГРА ОКОНЧЕНА! Счёт: %d\n", score)
	saveHighScore()
}
