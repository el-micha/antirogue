#![allow(dead_code)]

use tcod::colors::*;
use tcod::console::*;
use rand::Rng;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color {r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: Color = Color {r: 50, g: 50, b: 150};

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocking: bool,
    blocking_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {blocking: false, blocking_sight: false,}
    }

    pub fn wall() -> Self {
        Tile {blocking: true, blocking_sight: true,}
    }

}

type Map = Vec<Vec<Tile>>;
struct Game {
    map: Map,
}

fn make_map(player: &mut Entity) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    let mut rooms = vec![];
    for _ in 0..MAX_ROOMS {
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);
        let new_room = Rect::new(x, y, w, h);
        let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));
        if !failed {
            create_room(new_room, &mut map);
            let (new_x, new_y) = new_room.center();
            if rooms.is_empty() {
                // this is the first room only
                player.x = new_x;
                player.y = new_y;
            } else {
                // these are all other rooms
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rand::random() {
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                }
                else {
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            rooms.push(new_room);
        }
    }

    map
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let x = (self.x1 + self.x2) / 2;
        let y = (self.y1 + self.y2) / 2;
        (x, y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        (self.x1 <= other.x2)
        && (self.x2 >= other.x1)
        && (self.y1 <= other.y2)
        && (self.y2 >= other.y1)
    }
}

fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn in_order(x: i32, y: i32) -> (i32, i32) {
    if x > y {
        return (y, x);
    }
    (x, y)
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    let (x1, x2) = in_order(x1, x2);
    for x in x1..=x2 {
        map[x as usize][y as usize]= Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    let (y1, y2) = in_order(y1, y2);
    for y in y1..=y2 {
        map[x as usize][y as usize]= Tile::empty();
    }
}

struct Tcod {
    root: Root,
    console: Offscreen,
}

#[derive(Debug)]
struct Entity {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Entity {x, y, char, color}
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocking {
            self.x += dx;
            self.y += dy;
        }
    }

    pub fn draw(&self, console: &mut dyn Console) {
        console.set_default_foreground(self.color);
        console.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);
    let root = Root::initializer()
        .font("terminal16x16_gs_ro.png", FontLayout::AsciiInRow)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("a n t i - r o g u e")
        .init();
    let console = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let mut tcod = Tcod{root, console};


    let player = Entity::new(0, 0, '@', WHITE);
    let npc = Entity::new(55, 20, 'H', YELLOW);
    let mut entities = vec![player, npc];

    let game = Game {
        map: make_map(&mut entities[0]),
    };

    while !tcod.root.window_closed() {
        tcod.console.clear();

        render_all(&mut tcod, &game, &entities);

        tcod.root.flush();
        let player = &mut entities[0];
        let exit = handle_keys(&mut tcod, player, &game);
        if exit {
            break;
        }
    }

}

fn render_all(tcod: &mut Tcod, game: &Game, entities: &[Entity]) {
    for entity in entities {
        entity.draw(&mut tcod.console);
    }
    entities[0].draw(&mut tcod.console); // drawing player at the end
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].blocking_sight;
            if wall {
                tcod.console.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.console.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }
    blit(
            &tcod.console,
            (0, 0),
            (MAP_WIDTH, MAP_HEIGHT),
            &mut tcod.root,
            (0, 0),
            1.0,
            1.0,
        );
}

fn handle_keys(tcod: &mut Tcod, player: &mut Entity, game: &Game) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {code: Char, printable: 'w', ..} => player.move_by(0, -1, game),
        Key {code: Char, printable: 's', ..} => player.move_by(0, 1, game),
        Key {code: Char, printable: 'a', ..} => player.move_by(-1, 0, game),
        Key {code: Char, printable: 'd', ..} => player.move_by(1, 0, game),
        Key {code: Escape, ..} => return true, // exit game
        _ => {}
    }
    false
}


