use tcod::colors::*;
use tcod::console::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color {r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: Color = Color {r: 50, g: 50, b: 150};

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

fn make_map() -> Map {
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();
    map
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

    let game = Game {
        map: make_map(),
    };
    let player = Entity::new(SCREEN_WIDTH / 3, SCREEN_HEIGHT / 3, '@', WHITE);
    let npc = Entity::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, 'H', YELLOW);
    let mut entities = vec![player, npc];

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


