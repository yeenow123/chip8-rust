use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

const SCALE_FACTOR: u32 = 12;

pub struct GraphicsDevice {
    canvas: Canvas<Window>,
}

impl GraphicsDevice {
    pub fn new(sdl_context: &Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        let _window = video_subsystem
            .window("Keyboard", 64 * SCALE_FACTOR, 32 * SCALE_FACTOR)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = _window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        GraphicsDevice { canvas }
    }

    pub fn draw(&mut self, gfx: &[[u8; 64]; 32]) {
        for (y, row) in gfx.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let scaled_x = ((x) as u32 * SCALE_FACTOR) as i32;
                let scaled_y = ((y) as u32 * SCALE_FACTOR) as i32;

                if *cell != 0 {
                    self.canvas
                        .set_draw_color(pixels::Color::RGB(255, 255, 255));
                } else {
                    self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
                }
                let _ = self.canvas.fill_rect(Rect::new(
                    scaled_x,
                    scaled_y,
                    SCALE_FACTOR,
                    SCALE_FACTOR,
                ));
            }
        }
        self.canvas.present();
    }
}
