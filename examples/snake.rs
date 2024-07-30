use kontroll::{
    api::{self, Kontroll},
    utils,
};
use macroquad::prelude::*;
use std::collections::LinkedList;

// Example based on macroquad snake.rs example

type Point = (i16, i16);

struct Snake {
    head: Point,
    body: LinkedList<Point>,
    dir: Point,
}

#[macroquad::main("Snake")]
#[tokio::main]
async fn main() {
    // Macroquad is single-threaded by using a custom async executor calling next_frame().await
    // Therefore, we use tokio's runtime and block the thread in order to use the async api
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    const WIDTH: i16 = 12;
    const HEIGHT: i16 = 4;

    let mut snake = Snake {
        head: (0, 0),
        dir: (1, 0),
        body: LinkedList::new(),
    };
    let mut fruit: Point = (rand::gen_range(0, WIDTH), rand::gen_range(0, HEIGHT));
    let mut score = 0;
    let mut speed = 0.3;
    let mut last_update = get_time();
    let mut navigation_lock = false;
    let mut game_over = false;

    let up = (0, -1);
    let down = (0, 1);
    let right = (1, 0);
    let left = (-1, 0);

    let api = runtime.block_on(Kontroll::new(None)).unwrap();

    loop {
        if !game_over {
            if is_key_down(KeyCode::Right) && snake.dir != left && !navigation_lock {
                snake.dir = right;
                navigation_lock = true;
            } else if is_key_down(KeyCode::Left) && snake.dir != right && !navigation_lock {
                snake.dir = left;
                navigation_lock = true;
            } else if is_key_down(KeyCode::Up) && snake.dir != down && !navigation_lock {
                snake.dir = up;
                navigation_lock = true;
            } else if is_key_down(KeyCode::Down) && snake.dir != up && !navigation_lock {
                snake.dir = down;
                navigation_lock = true;
            }

            if get_time() - last_update > speed {
                last_update = get_time();
                snake.body.push_front(snake.head);
                snake.head = (snake.head.0 + snake.dir.0, snake.head.1 + snake.dir.1);

                // Wrap around logic
                if snake.head.0 < 0 {
                    snake.head.0 = WIDTH - 1;
                } else if snake.head.0 >= WIDTH {
                    snake.head.0 = 0;
                }
                if snake.head.1 < 0 {
                    snake.head.1 = HEIGHT - 1;
                } else if snake.head.1 >= HEIGHT {
                    snake.head.1 = 0;
                }

                if snake.head == fruit {
                    fruit = (rand::gen_range(0, WIDTH), rand::gen_range(0, HEIGHT));
                    score += 100;
                } else {
                    snake.body.pop_back();
                }

                for (x, y) in &snake.body {
                    if *x == snake.head.0 && *y == snake.head.1 {
                        game_over = true;
                    }
                }
                navigation_lock = false;
                let _ = runtime.block_on(api.set_rgb_all(0, 0, 0, 0));
            }
        }
        if !game_over {
            clear_background(LIGHTGRAY);

            let game_width = screen_width();
            let game_height = screen_height();
            let offset_x = (screen_width() - game_width) / 2. + 10.;
            let offset_y = (screen_height() - game_height) / 2. + 10.;
            let sq_width = (screen_width() - offset_x * 2.) / WIDTH as f32;
            let sq_height = (screen_height() - offset_y * 2.) / HEIGHT as f32;

            draw_rectangle(
                offset_x,
                offset_y,
                game_width - 20.,
                game_height - 20.,
                WHITE,
            );

            for i in 1..WIDTH {
                draw_line(
                    offset_x + sq_width * i as f32,
                    offset_y,
                    offset_x + sq_width * i as f32,
                    screen_height() - offset_y,
                    2.,
                    LIGHTGRAY,
                );
            }

            for i in 1..HEIGHT {
                draw_line(
                    offset_x,
                    offset_y + sq_height * i as f32,
                    screen_width() - offset_x,
                    offset_y + sq_height * i as f32,
                    2.,
                    LIGHTGRAY,
                );
            }

            draw_rectangle(
                offset_x + snake.head.0 as f32 * sq_width,
                offset_y + snake.head.1 as f32 * sq_height,
                sq_width,
                sq_height,
                DARKGREEN,
            );

            let _ = runtime.block_on(api.set_rgb_led(
                utils::pos_to_voyager(snake.head.0 as u16, snake.head.1 as u16),
                255,
                0,
                0,
                0,
            ));

            for (x, y) in &snake.body {
                draw_rectangle(
                    offset_x + *x as f32 * sq_width,
                    offset_y + *y as f32 * sq_height,
                    sq_width,
                    sq_height,
                    LIME,
                );

                let _ = runtime.block_on(api.set_rgb_led(
                    utils::pos_to_voyager(*x as u16, *y as u16),
                    80,
                    0,
                    0,
                    0,
                ));
            }

            draw_rectangle(
                offset_x + fruit.0 as f32 * sq_width,
                offset_y + fruit.1 as f32 * sq_height,
                sq_width,
                sq_height,
                GOLD,
            );

            let _ = runtime.block_on(api.set_rgb_led(
                utils::pos_to_voyager(fruit.0 as u16, fruit.1 as u16),
                255,
                255,
                224,
                0,
            ));

            draw_text(format!("SCORE: {score}").as_str(), 10., 20., 20., DARKGRAY);
        } else {
            clear_background(WHITE);
            let text = format!("Game Over. Score: {score}. Press [enter] to play again.");
            let font_size = 30.;
            let text_size = measure_text(text.as_str(), None, font_size as _, 1.0);
            speed = 1.0;
            draw_text(
                text.as_str(),
                screen_width() / 2. - text_size.width / 2.,
                screen_height() / 2. + text_size.height / 2.,
                font_size,
                DARKGRAY,
            );

            if get_time() - last_update > speed {
                last_update = get_time();
                let _ = runtime.block_on(api.restore_rgb_leds());
            }

            if is_key_down(KeyCode::Enter) {
                snake = Snake {
                    head: (0, 0),
                    dir: (1, 0),
                    body: LinkedList::new(),
                };
                fruit = (rand::gen_range(0, WIDTH), rand::gen_range(0, HEIGHT));
                score = 0;
                speed = 0.3;
                last_update = get_time();
                game_over = false;
            }
        }
        next_frame().await;
    }
}
