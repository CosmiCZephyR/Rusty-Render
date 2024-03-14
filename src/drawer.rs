use crate::camera::Camera;
use minifb::{ Key, Window };
use rusttype::{ point, Font, Scale };
use std::{ cell::RefCell, mem::swap, ops::{Add, Div, Mul}, rc::Rc, vec };

pub use crate::math::{ matrix4::Mat4, mesh::Mesh, vector4f::Vec4F };

// Заранее хочу предупредить, что следующий код проклят всеми программистами, которые его видели, при работе с ним рекомендуется
// 1. Позаботиться о наличии святой воды в непосредственной близости от вас
// 2. В случае малейшего сомнения в том, что что-то может пойти не так нужно срочно вызывать экзорциста
// 3. Перед началом разбирательства в коде сделать расклад карт таро, съездить к гадалке, прочитать гороскоп на ваш знак зодиака, получить нотальную карту
// и если хоть что-то из этого покажет неблагоприятные для вас известия не в коем случае не приступать
// 4. Помолиться всем известным вам богам и сделать жертвоприношение сатане, чтобы они все смиловались над вами и вашей грешной душой
// 5. Не бояться, то, что написано ниже чувствует страх, так же, если у вас есть открытые раны не стоит смотреть его, он чует запах крови за несколько километров
// 6. Не кодить в полнолуние, в 15:53 четверга и в 18:31 пятницы.

// В случае нужды рефакторинга нужно сходить и помолиться и оставить пулл реквест с предложениями или улучшениями.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub p: [Vec4F; 3],
    pub color: u32,
}

impl Triangle {
    pub fn average_z(&self) -> f32 {
        (self.p[0].z + self.p[1].z + self.p[2].z) / 3.0_f32
    }
}

impl Mul for Triangle {
    type Output = Triangle;

    fn mul(self, rhs: Self) -> Self::Output {
        Triangle {
            p: [self.p[0] * rhs.p[0], self.p[1] * rhs.p[1], self.p[2] * rhs.p[2]],
            color: self.color,
        }
    }
}

impl Mul<Mat4> for Triangle {
    type Output = Triangle;

    fn mul(self, rhs: Mat4) -> Self::Output {
        Triangle {
            p: [rhs * self.p[0], rhs * self.p[1], rhs * self.p[2]],
            color: self.color,
        }
    }
}

impl Div<f32> for Triangle {
    type Output = Triangle;

    fn div(self, rhs: f32) -> Self::Output {
        Triangle {
            p: [self.p[0] / rhs, self.p[1] / rhs, self.p[2] / rhs],
            color: self.color,
        }
    }
}

impl Add<Vec4F> for Triangle {
    type Output = Triangle;

    fn add(self, rhs: Vec4F) -> Self::Output {
        Triangle {
            p: [self.p[0] + rhs, self.p[1] + rhs, self.p[2] + rhs],
            color: self.color,
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            p: [Vec4F::default(), Vec4F::default(), Vec4F::default()],
            color: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Drawer {
    pub height: usize,
    pub width: usize,
    pub buffer: Vec<u32>,

    // Я хуй знает, на сколько это мнгого, но пусть будет
    window: Rc<RefCell<Window>>,
    mesh: Mesh,
    project_matrix: Mat4,
    theta: f32,
    pub camera: Camera,
}

impl Drawer {
    pub fn new(width: usize, height: usize, window: Rc<RefCell<Window>>) -> Drawer {
        Drawer {
            height,
            width,
            buffer: vec![0; width * height],
            mesh: Mesh::default(),
            project_matrix: Mat4::default(),
            theta: 0.0_f32,
            camera: Camera::default(),
            window,
        }
    }

    pub fn ready(&mut self) {
        self.mesh = self.mesh.parse_obj_file(r"src\objects\car.obj");

        self.project_matrix = self.camera.get_projection_matrix();
    }

    pub fn update(&mut self, elapsed_time: f32) {
        self.handle_input(elapsed_time);

        let mat_rot_z: Mat4 = Mat4::default().rotate_z(self.theta);
        let mat_rot_x: Mat4 = Mat4::default().rotate_x(self.theta * 0.5_f32);
        let mat_trans = Mat4::default().translate(0.0_f32, 0.0_f32, 5.0_f32);
        let mat_world: Mat4 = mat_rot_z * mat_rot_x * mat_trans;
        let mat_view = self.camera.get_view_matrix();

        let mut triangles_to_raster: Vec<Triangle> = Vec::new();

        for tri in self.mesh.tris.clone().iter_mut() {
            let mut tri_viewed: Triangle;
            let mut tri_transformed: Triangle = *tri * mat_world;

            let line1: Vec4F = tri_transformed.p[1] - tri_transformed.p[0];
            let line2: Vec4F = tri_transformed.p[2] - tri_transformed.p[0];
            let normal: Vec4F = line1.cross_product(&line2).normalize();

            let camera_ray = tri_transformed.p[0] - self.camera.position;

            if normal.dot_product(&camera_ray) < 0.0_f32 {
                let light_direction = Vec4F::new(0.0_f32, 1.0_f32, -1.0_f32).normalize();

                let dot_product = (0.1_f32).max(normal.dot_product(&light_direction));

                let col = Self::get_color(dot_product);
                tri_transformed.color = col;

                tri_viewed = tri_transformed * mat_view;
                tri_viewed.color = tri_transformed.color;

                let clipped: Vec<Triangle> = self.clip_against_plane(
                    &mut Vec4F::new(0.0_f32, 0.0_f32, 0.1_f32),
                    &mut Vec4F::new(0.0_f32, 0.0_f32, 1.0_f32),
                    &mut tri_viewed
                );

                self.project_triangle(&mut tri_viewed, clipped, &mut triangles_to_raster)
            }
        }

        triangles_to_raster.sort_by(|a, b| {
            b.average_z().partial_cmp(&a.average_z()).unwrap_or(std::cmp::Ordering::Equal)
        });

        self.fill(0, 0, self.width as i32, self.height as i32, 0);

        let planes = [
            (
                Vec4F { x: 0.0, y: 0.0, z: 0.0, ..Vec4F::default() },
                Vec4F { x: 0.0, y: 1.0, z: 0.0, ..Vec4F::default() },
            ),
            (
                Vec4F { x: 0.0, y: (self.height as f32) - 1.0, z: 0.0, ..Vec4F::default() },
                Vec4F { x: 0.0, y: -1.0, z: 0.0, ..Vec4F::default() },
            ),
            (
                Vec4F { x: 0.0, y: 0.0, z: 0.0, ..Vec4F::default() },
                Vec4F { x: 1.0, y: 0.0, z: 0.0, ..Vec4F::default() },
            ),
            (
                Vec4F { x: (self.width as f32) - 1.0, y: 0.0, z: 0.0, ..Vec4F::default() },
                Vec4F { x: -1.0, y: 0.0, z: 0.0, ..Vec4F::default() },
            ),
        ];

        for tri_to_raster in triangles_to_raster.iter() {
            let mut list_triangles: Vec<Triangle> = Vec::new();
            list_triangles.push(*tri_to_raster);

            for (ref mut plane_pos, ref mut plane_normal) in planes {
                let mut new_list_triangles = Vec::new();
                for test in list_triangles.drain(..) {
                    let tris_to_add = self.clip_against_plane(
                        plane_pos,
                        plane_normal,
                        &mut test.clone()
                    );
                    new_list_triangles.extend(tris_to_add);
                }
                list_triangles = new_list_triangles;
            }

            for t in list_triangles {
                self.fill_triangle_from(t);
                // self.draw_triangle_from(t);
            }
        }

        self.draw_string(
            10,
            100,
            format!("TRIANGLES: {}", triangles_to_raster.len()).as_str(),
            0xffffff
        );
    }

    fn project_triangle(&self, tri: &mut Triangle, clipped: Vec<Triangle>, tris_to_raster: &mut Vec<Triangle>) {
        for n in 0..clipped.len() {
            *tri = clipped[n] * self.project_matrix;
            tri.color = clipped[n].color;

            tri.p[0] = tri.p[0] / tri.p[0].w;
            tri.p[1] = tri.p[1] / tri.p[1].w;
            tri.p[2] = tri.p[2] / tri.p[2].w;

            tri.p[0].x *= -1.0_f32;
            tri.p[0].y *= -1.0_f32;
            tri.p[1].x *= -1.0_f32;
            tri.p[1].y *= -1.0_f32;
            tri.p[2].x *= -1.0_f32;
            tri.p[2].y *= -1.0_f32;

            let offset_view = Vec4F::new(1.0, 1.0, 0.0);

            *tri = *tri + offset_view;
            tri.p[0].x *= 0.5_f32 * (self.width as f32);
            tri.p[0].y *= 0.5_f32 * (self.height as f32);
            tri.p[1].x *= 0.5_f32 * (self.width as f32);
            tri.p[1].y *= 0.5_f32 * (self.height as f32);
            tri.p[2].x *= 0.5_f32 * (self.width as f32);
            tri.p[2].y *= 0.5_f32 * (self.height as f32);

            tris_to_raster.push(*tri);
        }
    }

    fn handle_input(&mut self, elapsed_time: f32) {
        self.window
            .borrow()
            .get_keys()
            .iter()
            .for_each(|key| {
                match key {
                    Key::Space => {
                        self.camera.position.y += 8.0 * elapsed_time;
                        if self.window.borrow().is_key_down(Key::LeftCtrl) {
                            self.camera.position.y += 16.0 * elapsed_time;
                        }
                    }
                    Key::LeftShift => {
                        self.camera.position.y -= 8.0 * elapsed_time;
                    }
                    Key::W => {
                        self.camera.position += self.camera.look_dir * (8.0 * elapsed_time);
                        if self.window.borrow().is_key_down(Key::LeftCtrl) {
                            self.camera.position +=
                                self.camera.look_dir * (8.0 * elapsed_time) * 2.0;
                        }
                    }
                    Key::S => {
                        self.camera.position -= self.camera.look_dir * (8.0 * elapsed_time);
                    }
                    Key::A => {
                        self.camera.yaw -= 2.0 * elapsed_time;
                    }
                    Key::D => {
                        self.camera.yaw += 2.0 * elapsed_time;
                    }
                    _ => {}
                }
            });
    }

    fn get_color(lum: f32) -> u32 {
        let pixel_bw = (13.0 * lum) as i32;

        let color = match pixel_bw {
            0 => 0x000000,
            1 => 0x151515,
            2 => 0x2a2a2a,
            3 => 0x3f3f3f,
            4 => 0x555555,
            5 => 0x6a6a6a,
            6 => 0x7f7f7f,
            7 => 0x949494,
            8 => 0xaaaaaa,
            9 => 0xbfbfbf,
            10 => 0xd4d4d4,
            11 => 0xe9e9e9,
            12 => 0xffffff,
            _ => 0x000000,
        };

        color
    }

    fn clip_against_plane(
        &mut self,
        plane_p: &mut Vec4F,
        plane_n: &mut Vec4F,
        in_tri: &mut Triangle
    ) -> Vec<Triangle> {
        *plane_n = plane_n.normalize();

        let (mut out_tri1, mut out_tri2) = (Triangle::default(), Triangle::default());

        let dist = |p: &mut Vec4F| {
            let _n = p.normalize();
            return plane_n.x * p.x + plane_n.y * p.y + plane_n.z * p.z - plane_n.dot_product(&plane_p);
        };

        let mut inside_points: [Vec4F; 3] = [Vec4F::default(); 3];
        let mut inside_points_count: i32 = 0;
        let mut outside_points: [Vec4F; 3] = [Vec4F::default(); 3];
        let mut outside_points_count: i32 = 0;

        let d0: f32 = dist(&mut in_tri.p[0]);
        let d1: f32 = dist(&mut in_tri.p[1]);
        let d2: f32 = dist(&mut in_tri.p[2]);

        //fucking fuck

        if d0 >= 0.0 {
            inside_points[inside_points_count as usize] = in_tri.p[0];
            inside_points_count += 1;
        } else {
            outside_points[outside_points_count as usize] = in_tri.p[0];
            outside_points_count += 1;
        }

        if d1 >= 0.0 {
            inside_points[inside_points_count as usize] = in_tri.p[1];
            inside_points_count += 1;
        } else {
            outside_points[outside_points_count as usize] = in_tri.p[1];
            outside_points_count += 1;
        }

        if d2 >= 0.0 {
            inside_points[inside_points_count as usize] = in_tri.p[2];
            inside_points_count += 1;
        } else {
            outside_points[outside_points_count as usize] = in_tri.p[2];
            outside_points_count += 1;
        }

        if inside_points_count == 0 {
            return vec![];
        }

        if inside_points_count == 3 {
            out_tri1 = *in_tri;

            return vec![out_tri1];
        }

        if inside_points_count == 1 && outside_points_count == 2 {
            out_tri1.color = in_tri.color;

            out_tri1.p[0] = inside_points[0];
            out_tri1.p[1] = Vec4F::intersects_plane(
                plane_p,
                plane_n,
                &inside_points[0],
                &outside_points[0]
            );
            out_tri1.p[2] = Vec4F::intersects_plane(
                plane_p,
                plane_n,
                &inside_points[0],
                &outside_points[1]
            );

            return vec![out_tri1];
        }

        if inside_points_count == 2 && outside_points_count == 1 {
            out_tri1.color = in_tri.color;
            out_tri2.color = in_tri.color;

            out_tri1.p[0] = inside_points[0];
            out_tri1.p[1] = inside_points[1];
            out_tri1.p[2] = Vec4F::intersects_plane(
                plane_p,
                plane_n,
                &inside_points[0],
                &outside_points[0]
            );

            out_tri2.p[0] = inside_points[1];
            out_tri2.p[1] = out_tri1.p[2];
            out_tri2.p[2] = Vec4F::intersects_plane(
                plane_p,
                plane_n,
                &inside_points[1],
                &outside_points[0]
            );

            return vec![out_tri1, out_tri2];
        }

        vec![]
    }

    pub fn draw_square(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, cal: u32) {
        self.draw_triangle(x1, y1, x2, y1, x2, y2, cal);
        self.draw_triangle(x1, y1, x1, y2, x2, y2, cal);
    }

    pub fn draw(&mut self, x: i32, y: i32, col: u32) {
        if x >= 0 && x < (self.width as i32) && y >= 0 && y < (self.height as i32) {
            self.buffer[(y * (self.width as i32) + x) as usize] = col;
        }
    }

    pub fn draw_circle(&mut self, xc: i32, yc: i32, r: i32) {
        let mut x: i32 = 0;
        let mut y: i32 = r;
        let mut p: i32 = 3 - 2 * r;
        if r == 0 {
            return;
        }

        while y >= x {
            self.draw(xc + x, yc + y, 0xffffff);
            self.draw(xc + y, yc + x, 0xffffff);
            self.draw(xc - y, yc + x, 0xffffff);
            self.draw(xc - x, yc + y, 0xffffff);
            self.draw(xc - x, yc - y, 0xffffff);
            self.draw(xc - y, yc - x, 0xffffff);
            self.draw(xc + y, yc - x, 0xffffff);
            self.draw(xc + x, yc - y, 0xffffff);
            if p < 0 {
                p += 4 * x + 6;
            } else {
                y -= 1;
                p += 4 * (x - y) + 10;
            }
            x += 1;
        }
    }

    pub fn draw_triangle(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        col: u32
    ) {
        self.draw_line(x1, y1, x2, y2, col);
        self.draw_line(x2, y2, x3, y3, col);
        self.draw_line(x3, y3, x1, y1, col);
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, col: u32) {
        let (mut x, mut y, dx, dy, dx1, dy1, mut px, mut py): (
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        );
        dx = x2 - x1;
        dy = y2 - y1;
        dx1 = dx.abs();
        dy1 = dy.abs();
        px = 2 * dy1 - dx1;
        py = 2 * dx1 - dy1;
        if dy1 <= dx1 {
            if dx >= 0 {
                x = x1;
                y = y1;
            } else {
                x = x2;
                y = y2;
            }
            self.draw(x, y, col);
            for _i in 0..dx1 {
                if px < 0 {
                    px = px + 2 * dy1;
                } else {
                    if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
                        y += 1;
                    } else {
                        y -= 1;
                    }
                    px = px + 2 * (dy1 - dx1);
                }
                x += 1;
                self.draw(x, y, col);
            }
        } else {
            if dy >= 0 {
                x = x1;
                y = y1;
            } else {
                x = x2;
                y = y2;
            }
            self.draw(x, y, col);
            for _i in 0..dy1 {
                if py <= 0 {
                    py = py + 2 * dx1;
                } else {
                    if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
                        x += 1;
                    } else {
                        x -= 1;
                    }
                    py = py + 2 * (dx1 - dy1);
                }
                y += 1;
                self.draw(x, y, col);
            }
        }
    }

    pub fn draw_triangle_from(&mut self, tri: Triangle) {
        self.draw_triangle(
            tri.p[0].x as i32,
            tri.p[0].y as i32,
            tri.p[1].x as i32,
            tri.p[1].y as i32,
            tri.p[2].x as i32,
            tri.p[2].y as i32,
            tri.color
        )
    }

    pub fn fill_triangle_from(&mut self, tri: Triangle) {
        self.fill_triangle(
            tri.p[0].x as i32,
            tri.p[0].y as i32,
            tri.p[1].x as i32,
            tri.p[1].y as i32,
            tri.p[2].x as i32,
            tri.p[2].y as i32,
            tri.color
        );
    }

    pub fn fill_triangle(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        col: u32
    ) {
        // count_calls(self);

        // Sort the points by y-coordinate
        let mut points = [
            (x1, y1),
            (x2, y2),
            (x3, y3),
        ];
        points.sort_by_key(|p| p.1);

        let (x1, y1) = points[0];
        let (x2, y2) = points[1];
        let (x3, y3) = points[2];

        // Calculate the slopes
        let slope_a = if y2 - y1 != 0 { ((x2 - x1) as f32) / ((y2 - y1) as f32) } else { 0.0 };
        let slope_b = if y3 - y1 != 0 { ((x3 - x1) as f32) / ((y3 - y1) as f32) } else { 0.0 };
        let slope_c = if y3 - y2 != 0 { ((x3 - x2) as f32) / ((y3 - y2) as f32) } else { 0.0 };

        // Draw the triangle
        for y in y1..=y2 {
            let xa = (x1 as f32) + slope_a * ((y - y1) as f32);
            let xb = (x1 as f32) + slope_b * ((y - y1) as f32);
            self.fill_line(xa.round() as i32, xb.round() as i32, y, col);
        }
        for y in y2..=y3 {
            let xa = (x2 as f32) + slope_c * ((y - y2) as f32);
            let xb = (x1 as f32) + slope_b * ((y - y1) as f32);
            self.fill_line(xa.round() as i32, xb.round() as i32, y, col);
        }
    }

    fn fill_line(&mut self, mut sx: i32, mut ex: i32, ny: i32, col: u32) {
        if sx > ex {
            swap(&mut sx, &mut ex);
        }
        for x in sx..=ex {
            self.draw(x, ny, col);
        }
    }

    fn swap(x: &mut i32, y: &mut i32) {
        swap(x, y);
    }

    #[allow(unused_assignments)]
    fn clip(&self, mut x: i32, mut y: i32) {
        if x < 0 {
            x = 0;
        }
        if x >= (self.width as i32) {
            x = (self.width as i32) - 1;
        }
        if y < 0 {
            y = 0;
        }
        if y >= (self.height as i32) {
            y = (self.height as i32) - 1;
        }
    }

    pub fn fill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, col: u32) {
        self.clip(x1, y1);
        self.clip(x2, y2);
        for x in x1..=x2 {
            for y in y1..=y2 {
                self.draw(x, y, col);
            }
        }
    }

    pub fn draw_string(&mut self, x: i32, y: i32, string: &str, col: u32) {
        let font: Font<'static> = Font::try_from_bytes(
            include_bytes!(r"assets\pixelfont.ttf") as &[u8]
        ).unwrap();
        let height: f32 = 20f32; // adjust as needed
        let scale = Scale {
            x: height,
            y: height,
        };
        let v_metrics = font.v_metrics(scale);
        let offset = point(x as f32, v_metrics.ascent + (y as f32));
        let iter = font.layout(string, scale, offset);

        for g in iter {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let v = v * (0xff as f32);
                    let x = x + (bb.min.x as u32);
                    let y = y + (bb.min.y as u32);
                    if v > 150.0 {
                        self.draw(x as i32, y as i32, col);
                    }
                });
            }
        }
    }

    fn get(&self, x: i32, y: i32) -> u32 {
        self.buffer[(y as usize) * self.width + (x as usize)]
    }
}
