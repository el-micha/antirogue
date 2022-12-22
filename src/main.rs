use tcod::colors::*;
use tcod::console::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

struct Tcod {
    root: Root,
    console: Offscreen,
}

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

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
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
    let console = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut tcod = Tcod{root, console};

    let player = Entity::new(SCREEN_WIDTH / 3, SCREEN_HEIGHT / 3, '@', WHITE);
    let npc = Entity::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, 'H', YELLOW);
    let mut entities = vec![player, npc];

    while !tcod.root.window_closed() {
        tcod.console.clear();

        for entity in &entities {
            entity.draw(&mut tcod.console);
        }

        blit(
            &tcod.console,
            (0, 0),
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            &mut tcod.root,
            (0, 0),
            1.0,
            1.0,
        );


        tcod.root.flush();
        let player = &mut entities[0];
        let exit = handle_keys(&mut tcod, player);
        if exit {
            break;
        }
    }

}

fn handle_keys(tcod: &mut Tcod, player: &mut Entity) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {code: Char, printable: 'w', ..} => player.move_by(0, -1),
        Key {code: Char, printable: 's', ..} => player.move_by(0, 1),
        Key {code: Char, printable: 'a', ..} => player.move_by(-1, 0),
        Key {code: Char, printable: 'd', ..} => player.move_by(1, 0),
        Key {code: Escape, ..} => return true, // exit game
        _ => {}
    }
    false
}


