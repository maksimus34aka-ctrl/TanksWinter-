# tanks_winter.py — Танки (зима) на Python

import os
import time
import random
import sys
from collections import deque

try:
    import keyboard
    HAS_KEYBOARD = True
except ImportError:
    HAS_KEYBOARD = False
    print("Установите keyboard: pip install keyboard")
    sys.exit(1)

# Параметры поля
W, H = 30, 20
EMPTY = ' '
WALL = '#'
SNOW = '.'
ICE = '~'
TANK = '▲'
ENEMY = 'E'
BULLET = '*'
BONUS = 'B'

# Направления
DIRS = {'up': (0, -1), 'down': (0, 1), 'left': (-1, 0), 'right': (1, 0)}

class Tank:
    def __init__(self, x, y, direction='up', is_player=True):
        self.x = x
        self.y = y
        self.dir = direction
        self.is_player = is_player
        self.hp = 3
        self.speed = 1
        self.cooldown = 0

    def get_symbol(self):
        if self.is_player:
            return TANK
        return ENEMY

class Game:
    def __init__(self):
        self.width = W
        self.height = H
        self.field = [[EMPTY for _ in range(self.width)] for _ in range(self.height)]
        self.player = Tank(self.width//2, self.height//2, 'up', True)
        self.enemies = []
        self.bullets = []
        self.bonuses = []
        self.score = 0
        self.high_score = self.load_high_score()
        self.lives = 3
        self.level = 1
        self.game_over = False
        self.paused = False
        self.generate_field()
        self.spawn_enemy()

    def load_high_score(self):
        try:
            with open('highscore.txt', 'r') as f:
                return int(f.read())
        except:
            return 0

    def save_high_score(self):
        with open('highscore.txt', 'w') as f:
            f.write(str(self.high_score))

    def generate_field(self):
        # Очистка
        for y in range(self.height):
            for x in range(self.width):
                self.field[y][x] = EMPTY
        # Стены по периметру
        for x in range(self.width):
            self.field[0][x] = WALL
            self.field[self.height-1][x] = WALL
        for y in range(self.height):
            self.field[y][0] = WALL
            self.field[y][self.width-1] = WALL
        # Снег и лёд (случайно)
        for y in range(2, self.height-2):
            for x in range(2, self.width-2):
                r = random.random()
                if r < 0.3:
                    self.field[y][x] = SNOW
                elif r < 0.4:
                    self.field[y][x] = ICE
        # Стены внутри (10%)
        for y in range(2, self.height-2):
            for x in range(2, self.width-2):
                if random.random() < 0.08:
                    self.field[y][x] = WALL

    def spawn_enemy(self):
        # Появление врага на краю
        side = random.randint(0, 3)
        if side == 0:  # верх
            x = random.randint(2, self.width-3)
            y = 1
        elif side == 1:  # низ
            x = random.randint(2, self.width-3)
            y = self.height-2
        elif side == 2:  # лево
            x = 1
            y = random.randint(2, self.height-3)
        else:  # право
            x = self.width-2
            y = random.randint(2, self.height-3)
        # Проверяем, что место свободно
        if self.field[y][x] == EMPTY and not self.is_occupied(x, y):
            enemy = Tank(x, y, random.choice(['up','down','left','right']), False)
            self.enemies.append(enemy)

    def is_occupied(self, x, y):
        for obj in self.enemies + self.bullets:
            if obj.x == x and obj.y == y:
                return True
        if self.player.x == x and self.player.y == y:
            return True
        return False

    def move_tank(self, tank, dx, dy):
        nx, ny = tank.x + dx, tank.y + dy
        if nx < 1 or nx >= self.width-1 or ny < 1 or ny >= self.height-1:
            return False
        if self.field[ny][nx] == WALL:
            return False
        if self.is_occupied(nx, ny):
            return False
        tank.x, tank.y = nx, ny
        return True

    def shoot(self, tank):
        if tank.cooldown > 0:
            return
        dx, dy = DIRS[tank.dir]
        bx, by = tank.x + dx, tank.y + dy
        if bx < 1 or bx >= self.width-1 or by < 1 or by >= self.height-1:
            return
        if self.field[by][bx] == WALL:
            # стена разрушается
            self.field[by][bx] = EMPTY
            return
        self.bullets.append({'x': bx, 'y': by, 'dx': dx, 'dy': dy, 'owner': tank.is_player})
        tank.cooldown = 5

    def update(self):
        if self.game_over or self.paused:
            return
        # Обновление врагов
        for enemy in self.enemies[:]:
            # Движение врага (случайное)
            if random.random() < 0.1:
                d = random.choice(['up','down','left','right'])
                dx, dy = DIRS[d]
                if self.move_tank(enemy, dx, dy):
                    enemy.dir = d
            # Стрельба врага (редко)
            if random.random() < 0.02:
                self.shoot(enemy)
        # Обновление пуль
        for bullet in self.bullets[:]:
            bullet['x'] += bullet['dx']
            bullet['y'] += bullet['dy']
            # Проверка столкновений
            if bullet['x'] < 1 or bullet['x'] >= self.width-1 or bullet['y'] < 1 or bullet['y'] >= self.height-1:
                self.bullets.remove(bullet)
                continue
            if self.field[bullet['y']][bullet['x']] == WALL:
                self.field[bullet['y']][bullet['x']] = EMPTY
                self.bullets.remove(bullet)
                continue
            # Попадание в игрока
            if bullet['owner'] == False and bullet['x'] == self.player.x and bullet['y'] == self.player.y:
                self.lives -= 1
                self.bullets.remove(bullet)
                if self.lives <= 0:
                    self.game_over = True
                    if self.score > self.high_score:
                        self.high_score = self.score
                        self.save_high_score()
                continue
            # Попадание во врага
            if bullet['owner'] == True:
                for enemy in self.enemies[:]:
                    if bullet['x'] == enemy.x and bullet['y'] == enemy.y:
                        self.enemies.remove(enemy)
                        self.bullets.remove(bullet)
                        self.score += 1
                        if self.score % 5 == 0:
                            self.level += 1
                            # добавить бонус
                            self.bonuses.append({'x': enemy.x, 'y': enemy.y, 'type': 'life'})
                        break
        # Обновление бонусов
        for bonus in self.bonuses[:]:
            # Проверка сбора игроком
            if bonus['x'] == self.player.x and bonus['y'] == self.player.y:
                if bonus['type'] == 'life':
                    self.lives += 1
                self.bonuses.remove(bonus)
        # Появление новых врагов (макс 5)
        if len(self.enemies) < min(3 + self.level, 8):
            if random.random() < 0.01 * self.level:
                self.spawn_enemy()
        # Уменьшение кулдаунов
        if self.player.cooldown > 0:
            self.player.cooldown -= 1
        for enemy in self.enemies:
            if enemy.cooldown > 0:
                enemy.cooldown -= 1

    def draw(self):
        os.system('cls' if os.name == 'nt' else 'clear')
        print(f"🎮 TanksWinter  |  Счёт: {self.score}  |  Жизни: {self.lives}  |  Уровень: {self.level}  |  Рекорд: {self.high_score}")
        if self.paused:
            print("⏸ ПАУЗА")
        # Верхняя граница
        print('+' + '-'*self.width + '+')
        for y in range(self.height):
            line = '|'
            for x in range(self.width):
                ch = self.field[y][x]
                # Игрок
                if x == self.player.x and y == self.player.y:
                    ch = TANK
                # Враги
                for enemy in self.enemies:
                    if enemy.x == x and enemy.y == y:
                        ch = ENEMY
                        break
                # Пули
                for bullet in self.bullets:
                    if bullet['x'] == x and bullet['y'] == y:
                        ch = BULLET
                        break
                # Бонусы
                for bonus in self.bonuses:
                    if bonus['x'] == x and bonus['y'] == y:
                        ch = BONUS
                        break
                line += ch
            line += '|'
            print(line)
        print('+' + '-'*self.width + '+')
        print("Управление: WASD - движение, Пробел - стрельба, P - пауза, Q - выход")

    def handle_input(self):
        if keyboard.is_pressed('w'): return 'up'
        if keyboard.is_pressed('s'): return 'down'
        if keyboard.is_pressed('a'): return 'left'
        if keyboard.is_pressed('d'): return 'right'
        if keyboard.is_pressed('space'): return 'shoot'
        if keyboard.is_pressed('p'): return 'pause'
        if keyboard.is_pressed('q'): return 'quit'
        return None

    def run(self):
        last_time = time.time()
        while not self.game_over:
            self.draw()
            cmd = self.handle_input()
            if cmd == 'quit':
                break
            if cmd == 'pause':
                self.paused = not self.paused
            elif cmd in DIRS:
                dx, dy = DIRS[cmd]
                if self.move_tank(self.player, dx, dy):
                    self.player.dir = cmd
            elif cmd == 'shoot':
                self.shoot(self.player)
            # Обновление игры
            if time.time() - last_time > 0.1:  # 10 FPS
                self.update()
                last_time = time.time()
            time.sleep(0.02)
        self.save_high_score()
        print("ИГРА ОКОНЧЕНА! Ваш счёт:", self.score)

if __name__ == "__main__":
    game = Game()
    game.run()
