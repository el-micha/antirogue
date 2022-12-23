#![allow(dead_code)]

use tcod::colors::*;
use tcod::console::*;
use rand::Rng;
use tcod::input::is_cursor_visible;
use tcod::map::{FovAlgorithm, Map as FovMap};

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

const COLOR_DARK_WALL: Color = Color {r: 0, g: 0, b: 100};
const COLOR_LIGHT_WALL: Color = Color {r: 130, g: 110, b: 50};
const COLOR_DARK_GROUND: Color = Color {r: 50, g: 50, b: 150};
const COLOR_LIGHT_GROUND: Color = Color {r: 200, g: 180, b: 50};

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const PLAYER: usize = 0; // as index into entity vector

struct Tcod {
    root: Root,
    console: Offscreen,
    fov: FovMap,
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

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocking: bool,
    blocking_sight: bool,
    explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {blocking: false, blocking_sight: false, explored: false,}
    }

    pub fn wall() -> Self {
        Tile {blocking: true, blocking_sight: true, explored: false,}
    }

}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
    fov_recompute: bool,
}

impl Game {
    fn reset_fov(&mut self) {
        self.fov_recompute = false;
    }

    fn set_recalculate_fov(&mut self) {
        self.fov_recompute = true;
    }
}

fn make_map(entities: &mut Vec<Entity>) -> Map {
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
            place_entities(new_room, &map, entities);
            let (new_x, new_y) = new_room.center();
            if rooms.is_empty() {
                // this is the first room only
                let player = &mut entities[0];
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


fn place_entities(room: Rect, map: &Map, entities: &mut Vec<Entity>) {
    let num_creatures = rand::thread_rng().gen_range(1, 4);
    for _ in 0..num_creatures {
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);
        if is_blocked(x, y, map, entities) {
            continue;
        }
        let roll = rand::random::<f32>();
        let creature = if roll < 0.2 {
            Entity::new(x, y, 'U', "Unicorn", DARKER_CYAN, true, true)
        } else if roll < 0.6 {
            Entity::new(x, y, 'f', "Fairy", MAGENTA, true, true)
        } else {
            Entity::new(x, y, 'e', "Elf", DARK_RED, true, true)
        };
        entities.push(creature);
    }
}

fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
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

fn in_order(x: i32, y: i32) -> (i32, i32) {
    if x > y {
        return (y, x);
    }
    (x, y)
}

fn is_blocked(x: i32, y: i32, map: &Map, entities: &[Entity]) -> bool {
    if map[x as usize][y as usize].blocking {
        return true;
    }
    entities.iter().any(|entity| entity.blocking && entity.pos() == (x, y))
}

#[derive(Debug)]
struct Entity {
    x: i32,
    y: i32,
    char: char,
    color: Color,
    name: String,
    blocking: bool,
    alive: bool,
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color, blocking: bool, alive: bool) -> Self {
        Entity {
            x: x,
            y: y,
            char: char,
            name: name.into(),
            color: color,
            blocking: blocking,
            alive: alive,
        }
    }

    pub fn draw(&self, console: &mut dyn Console) {
        console.set_default_foreground(self.color);
        console.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}
// belongs to Entity, but not a member due to ownership issue
fn move_by(subject_index: usize, dx: i32, dy: i32, game: &mut Game, entities: &mut [Entity]) {
    let (x, y) = entities[subject_index].pos();
    if !is_blocked(x + dx, y + dy, &game.map, entities) {
        entities[subject_index].set_pos(x + dx, y + dy);
        game.set_recalculate_fov();
    }
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);
    let root = Root::initializer()
        .font("terminal16x16_gs_ro.png", FontLayout::AsciiInRow)
        //.font("consolas10x10_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("a n t i - r o g u e")
        .init();
    let mut tcod = Tcod{
        root,
        console: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };


    let player = Entity::new(0, 0, '@', "Player", WHITE, true, true);
    let mut entities = vec![player];

    let mut game = Game {
        map: make_map(&mut entities),
        fov_recompute: true,
    };

    // populate FOV map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(x, y, !game.map[x as usize][y as usize].blocking_sight, !game.map[x as usize][y as usize].blocking);
        }
    }
    // ================================= MAIN LOOP =================================
    while !tcod.root.window_closed() {

        tcod.console.clear();
        render_all(&mut tcod, &mut game, &entities);
        tcod.root.flush();
        game.reset_fov();

        let exit = handle_keys(&mut tcod, &mut entities, &mut game);
        if exit {
            break;
        }

    }

}

fn render_all(tcod: &mut Tcod, game: &mut Game, entities: &[Entity]) {
    if game.fov_recompute {
        let player = &entities[PLAYER];
        tcod.fov.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    for entity in entities {
        if tcod.fov.is_in_fov(entity.x, entity.y) {
            entity.draw(&mut tcod.console);
        }
    }
    entities[0].draw(&mut tcod.console); // draw player again at the end

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].blocking_sight;
            let color = match (visible, wall) {
                // outside fov
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // inside fov
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }
            if *explored {
                tcod.console.set_char_background(x, y, color, BackgroundFlag::Set);
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

fn handle_keys(tcod: &mut Tcod, entities: &mut [Entity], game: &mut Game) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {code: Char, printable: 'w', ..} => move_by(PLAYER,0, -1, game, entities),
        Key {code: Char, printable: 's', ..} => move_by(PLAYER,0, 1, game, entities),
        Key {code: Char, printable: 'a', ..} => move_by(PLAYER,-1, 0, game, entities),
        Key {code: Char, printable: 'd', ..} => move_by(PLAYER,1, 0, game, entities),
        Key {code: Escape, ..} => return true, // exit game
        _ => {}
    }
    false
}


