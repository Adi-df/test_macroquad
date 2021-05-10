use hecs::{Entity, World};
use macroquad::prelude::*;

struct FrameCounter;
struct ScoreCounter;
struct Counter(u64);

struct SpaceShip;
struct FallingEnnemy;
struct EnnemyFactory;

struct Speed(u16);

#[derive(Clone)]
struct CellPosition(u16, u16);
struct CellSize(u16, u16);
struct CellRender(Color);

struct TextContent(String);
struct TextPosition(f32, f32);
struct TextRender(f32, Color);

fn render(world: &mut World, cell_size: (f32, f32)) {
    clear_background(BLACK);

    // Rects
    for (_, (render, pos, size)) in world.query_mut::<(&CellRender, &CellPosition, &CellSize)>() {
        draw_rectangle(
            pos.0 as f32 * cell_size.0,
            pos.1 as f32 * cell_size.1,
            size.0 as f32 * cell_size.0,
            size.1 as f32 * cell_size.1,
            render.0,
        );
    }

    //Text
    for (_, (render, text, pos)) in world.query_mut::<(&TextRender, &TextContent, &TextPosition)>()
    {
        draw_text(&text.0, pos.0, pos.1, render.0, render.1);
    }
}

fn create_new_world(grid_size: (u16, u16)) -> (World, (Entity, Entity, Entity, Entity)) {
    let mut world = World::new();

    let frame_counter = world.spawn((FrameCounter, Counter(0)));
    let score_counter = world.spawn((
        ScoreCounter,
        Counter(0),
        TextContent(String::from("Score: 0")),
        TextPosition(10., 30.),
        TextRender(30., WHITE),
    ));

    let spaceship = world.spawn((
        SpaceShip,
        CellRender(RED),
        CellPosition(grid_size.0 / 2, grid_size.1 - 2),
        CellSize(1, 1),
    ));

    let factory = world.spawn((EnnemyFactory, Counter(0)));

    (world, (spaceship, factory, frame_counter, score_counter))
}

#[macroquad::main("Spaceship")]
async fn main() {
    rand::srand((get_time() * 100000.).floor() as u64);
    loop {
        if !mainloop().await {
            break;
        }
    }
}

async fn mainloop() -> bool {
    let grid_size = (20, 20);
    let cell_size = (
        (screen_width() / grid_size.0 as f32).floor(),
        (screen_height() / grid_size.1 as f32).floor(),
    );

    // Main loop
    let score = {
        let (mut world, (spaceship, factory, frame_counter, score_counter)) =
            create_new_world(grid_size);

        loop {
            // Increment frame counts
            let frame = {
                for (_, (_, count)) in world.query_mut::<(&FrameCounter, &mut Counter)>() {
                    if count.0 != u64::MAX {
                        count.0 += 1
                    } else {
                        count.0 = 0
                    }
                }

                world.query_one_mut::<&Counter>(frame_counter).unwrap().0
            };

            // Spaceship Move
            {
                let spaceship_pos = world.query_one_mut::<&mut CellPosition>(spaceship).unwrap();

                if is_key_pressed(KeyCode::Right) && spaceship_pos.0 < grid_size.0 - 1 {
                    spaceship_pos.0 += 1;
                } else if is_key_pressed(KeyCode::Left) && spaceship_pos.0 > 0 {
                    spaceship_pos.0 -= 1;
                } else if is_key_pressed(KeyCode::Right) && spaceship_pos.0 == grid_size.0 - 1 {
                    spaceship_pos.0 = 0;
                } else if is_key_pressed(KeyCode::Left) && spaceship_pos.0 == 0 {
                    spaceship_pos.0 = grid_size.0 - 1;
                }
            }

            // Ennemy Factory
            {
                let mut countdown = world
                    .query_one_mut::<&mut Counter>(factory)
                    .unwrap()
                    .0
                    .clone();

                if countdown == 0 {
                    countdown = 120;
                    let speed = 10;
                    let hole = rand::gen_range(0, grid_size.0 as u32) as u16;

                    for x in 0..=grid_size.0 {
                        if x == hole {
                            continue;
                        }
                        world.spawn((
                            FallingEnnemy,
                            CellRender(BLUE),
                            CellPosition(x, 0),
                            CellSize(1, 1),
                            Speed(speed),
                        ));
                    }
                }
                countdown -= 1;

                world.query_one_mut::<&mut Counter>(factory).unwrap().0 = countdown;
            }

            //Kill out ennemy
            {
                let mut to_kill = Vec::new();

                for (ennemy, (_, pos)) in world.query_mut::<(&FallingEnnemy, &CellPosition)>() {
                    if pos.1 == grid_size.1 - 1 {
                        to_kill.push(ennemy);
                    }
                }

                if to_kill.len() > 0 {
                    world
                        .query_one_mut::<&mut Counter>(score_counter)
                        .unwrap()
                        .0 += 1;
                }
                for ennemy in to_kill {
                    world.despawn(ennemy).unwrap();
                }
            }

            // Falling ennemy Move
            {
                for (_, (_, pos, speed)) in
                    world.query_mut::<(&FallingEnnemy, &mut CellPosition, &Speed)>()
                {
                    if (frame % speed.0 as u64) == 0 {
                        pos.1 += 1;
                    }
                }
            }

            // Update score
            {
                let (score_text, score) = world
                    .query_one_mut::<(&mut TextContent, &Counter)>(score_counter)
                    .unwrap();
                score_text.0 = format!("Score: {}", score.0);
            }

            render(&mut world, cell_size);

            let gameover = {
                let spaceship_pos = world
                    .query_one_mut::<&CellPosition>(spaceship)
                    .unwrap()
                    .clone();
                world
                    .query_mut::<(&FallingEnnemy, &CellPosition)>()
                    .into_iter()
                    .any(|(_, (_, pos))| pos.0 == spaceship_pos.0 && pos.1 == spaceship_pos.1)
            };
            if !gameover {
                next_frame().await
            } else {
                break;
            }
        }

        world
            .query_one_mut::<&Counter>(score_counter)
            .unwrap()
            .0
            .clone()
    };

    // Game Over
    {
        // 0 for restart, 1 for quit
        let mut game_continue = true;
        loop {
            clear_background(BLACK);

            let message = format!("Game Over!");
            let size = measure_text(&message, None, 30, 1.);
            draw_text(
                &message,
                screen_width() / 2. - size.width / 2.,
                screen_height() / 2. - size.height / 2.,
                30.,
                WHITE,
            );
            let message = format!("Press [SPACE] to continue | [ENTER] to quit...");
            let size = measure_text(&message, None, 30, 1.);
            draw_text(
                &message,
                screen_width() / 2. - size.width / 2.,
                screen_height() / 2. + size.height,
                30.,
                WHITE,
            );
            let message = format!("Score: {}", score);
            let size = measure_text(&message, None, 30, 1.);
            draw_text(
                &message,
                screen_width() / 2. - size.width / 2.,
                screen_height() / 2. + size.height * 4.,
                30.,
                WHITE,
            );

            if is_key_pressed(KeyCode::Space) {
                break;
            } else if is_key_pressed(KeyCode::Enter) {
                game_continue = false;
                break;
            }

            next_frame().await
        }
        game_continue
    }
}
