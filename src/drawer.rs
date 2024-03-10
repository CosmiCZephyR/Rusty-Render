use minifb::{Key, Window};
use rand::Rng;
use rusttype::{point, Font, Scale};
use std::{cell::RefCell, collections::VecDeque, mem::swap, rc::Rc, vec};

pub use crate::math::{matrix4::Mat4, mesh::Mesh, vector4f::Vec4F};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub p: [Vec4F; 3],
    pub color: u32,
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            p: [
                Vec4F {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    ..Vec4F::default()
                },
                Vec4F {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    ..Vec4F::default()
                },
                Vec4F {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    ..Vec4F::default()
                },
            ],
            color: 0,
        }
    }
}

// lazy_static! {
//     pub static ref COUNTER: Mutex<u32> = Mutex::new(0);
//     pub static ref START_TIME: Mutex<Instant> = Mutex::new(Instant::now());
// }

// fn count_calls(_drawer: &mut Drawer) {
//     *COUNTER.lock().unwrap() += 1;
//
//     let elapse = START_TIME.lock().unwrap().elapsed();
//     if elapse >= Duration::from_secs(1) {
//         *COUNTER.lock().unwrap() = 0;
//         *START_TIME.lock().unwrap() = Instant::now();
//     }
// }

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
    camera: Vec4F,
    pub look_dir: Vec4F,
    pub pitch: f32,
    pub yaw: f32,
}

impl Drawer {
    pub fn new(width: usize, height: usize, window: Rc<RefCell<Window>>) -> Drawer {
        Drawer {
            height,
            width,
            buffer: vec![0; width * height],
            mesh: Mesh::default(),
            project_matrix: Mat4::default(),
            theta: 0.0,
            camera: Vec4F::default(),
            look_dir: Vec4F::default(),
            window,
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    pub fn ready(&mut self) -> (f32, f32, f32, f32) {
        self.mesh = self
            .mesh
            .parse_obj_file(r"E:\Projects\rusty-3d\src\objects\ship.obj");

        let near: f32 = 0.05;
        let far: f32 = 4000.0;
        let fov_deg: f32 = 75.0;
        let aspect_ratio = self.height as f32 / self.width as f32;

        self.project_matrix = Mat4::project(fov_deg, aspect_ratio, near, far);

        (near, far, fov_deg, aspect_ratio)
    }

    pub fn update(&mut self, elapsed_time: f32) {
        self.handle_input(elapsed_time);

        let mut mat_rot_z: Mat4 = Mat4::default();
        let mut mat_rot_x: Mat4 = Mat4::default();

        mat_rot_z = mat_rot_z.rotate_z(self.theta);
        mat_rot_x = mat_rot_x.rotate_x(self.theta * 0.5);

        let mut mat_trans = Mat4::default();
        mat_trans = mat_trans.translate(0.0, 0.0, 5.0);

        let mut mat_world: Mat4;
        mat_world = mat_rot_z * mat_rot_x;
        mat_world = mat_world * mat_trans;

        let up = Vec4F {
            x: 0.0,
            y: 1.0,
            z: 0.0,
            ..Vec4F::default()
        };

        let target = Vec4F {
            x: 0.0,
            y: 0.0,
            z: 1.0,
            ..Vec4F::default()
        };

        let mut mat_camera_rot = Mat4::default();
        mat_camera_rot = mat_camera_rot.rotate_y(self.yaw);
        self.look_dir = mat_camera_rot * target;
        let target = self.camera + self.look_dir;

        let mat_camera = Mat4::point_at(self.camera, target, up);

        let mat_view = mat_camera.quick_inverse();

        let mut triangles_to_raster: Vec<Triangle> = Vec::new();

        for tri in self.mesh.tris.clone().iter_mut() {
            let mut tri_projected = Triangle::default();
            let mut tri_transformed = Triangle::default();
            let mut tri_viewed = Triangle::default();

            tri_transformed.p[0] = mat_world * tri.p[0];
            tri_transformed.p[1] = mat_world * tri.p[1];
            tri_transformed.p[2] = mat_world * tri.p[2];

            let mut normal: Vec4F;
            let line1: Vec4F;
            let line2: Vec4F;

            line1 = tri_transformed.p[1] - tri_transformed.p[0];
            line2 = tri_transformed.p[2] - tri_transformed.p[0];

            normal = line1.cross_product(&line2);
            normal = normal.normalize();

            let camera_ray = tri_transformed.p[0] - self.camera;

            if normal.dot_product(&camera_ray) < 0.0 {
                let mut light_direction = Vec4F {
                    x: 0.0,
                    y: 1.0,
                    z: -1.0,
                    ..Vec4F::default()
                };
                light_direction = light_direction.normalize();

                let dot_product = 0.1f32.max(normal.dot_product(&light_direction));

                let col = Self::get_color(dot_product);
                tri_transformed.color = col;

                tri_viewed.p[0] = mat_view * tri_transformed.p[0];
                tri_viewed.p[1] = mat_view * tri_transformed.p[1];
                tri_viewed.p[2] = mat_view * tri_transformed.p[2];
                tri_viewed.color = tri_transformed.color;

                let clipped: Vec<Triangle>;
                clipped = self.clip_against_plane(
                    &mut Vec4F {
                        x: 0.0,
                        y: 0.0,
                        z: 0.1,
                        ..Vec4F::default()
                    },
                    &mut Vec4F {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                        ..Vec4F::default()
                    },
                    &mut tri_viewed,
                );

                for n in 0..clipped.len() {
                    tri_projected.p[0] = self.project_matrix * clipped[n].p[0];
                    tri_projected.p[1] = self.project_matrix * clipped[n].p[1];
                    tri_projected.p[2] = self.project_matrix * clipped[n].p[2];
                    tri_projected.color = clipped[n].color;

                    tri_projected.p[0] = tri_projected.p[0] / tri_projected.p[0].w;
                    tri_projected.p[1] = tri_projected.p[1] / tri_projected.p[1].w;
                    tri_projected.p[2] = tri_projected.p[2] / tri_projected.p[2].w;

                    tri_projected.p[0].x *= -1.0;
                    tri_projected.p[0].y *= -1.0;
                    tri_projected.p[1].x *= -1.0;
                    tri_projected.p[1].y *= -1.0;
                    tri_projected.p[2].x *= -1.0;
                    tri_projected.p[2].y *= -1.0;

                    let offset_view = Vec4F {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0,
                        ..Vec4F::default()
                    };
                    tri_projected.p[0] = tri_projected.p[0] + offset_view;
                    tri_projected.p[1] = tri_projected.p[1] + offset_view;
                    tri_projected.p[2] = tri_projected.p[2] + offset_view;
                    tri_projected.p[0].x *= 0.5 * self.width as f32;
                    tri_projected.p[0].y *= 0.5 * self.height as f32;
                    tri_projected.p[1].x *= 0.5 * self.width as f32;
                    tri_projected.p[1].y *= 0.5 * self.height as f32;
                    tri_projected.p[2].x *= 0.5 * self.width as f32;
                    tri_projected.p[2].y *= 0.5 * self.height as f32;

                    triangles_to_raster.push(tri_projected);
                }
            }
        }

        triangles_to_raster.sort_by(|a, b| {
            let z1 = (a.p[0].z + a.p[1].z + a.p[2].z) / 3.0;
            let z2 = (b.p[0].z + b.p[1].z + b.p[2].z) / 3.0;
            z2.partial_cmp(&z1).unwrap_or(std::cmp::Ordering::Equal)
        });

        self.fill(0, 0, self.width as i32, self.height as i32, 0);

        for tri_to_raster in triangles_to_raster.iter() {
            let mut list_triangles: VecDeque<Triangle> = VecDeque::new();
            list_triangles.push_back(*tri_to_raster);

            // println!("Triangles: {:?}", list_triangles);
            let mut new_triangles = 1;

            for p in 1..=4 {
                let mut tris_to_add: Vec<Triangle> = vec![];
                while new_triangles > 0 {
                    let mut test = *list_triangles.front().unwrap();
                    list_triangles.pop_front();
                    new_triangles -= 1;

                    match p {
                        1 => {
                            tris_to_add = self.clip_against_plane(
                                &mut Vec4F {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut Vec4F {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut test,
                            );
                        }
                        2 => {
                            tris_to_add = self.clip_against_plane(
                                &mut Vec4F {
                                    x: 0.0,
                                    y: self.height as f32 - 1.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut Vec4F {
                                    x: 0.0,
                                    y: -1.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut test,
                            );
                        }
                        3 => {
                            tris_to_add = self.clip_against_plane(
                                &mut Vec4F {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut Vec4F {
                                    x: 1.0,
                                    y: 0.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut test,
                            );
                        }
                        4 => {
                            tris_to_add = self.clip_against_plane(
                                &mut Vec4F {
                                    x: self.width as f32 - 1.0,
                                    y: 0.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut Vec4F {
                                    x: -1.0,
                                    y: 0.0,
                                    z: 0.0,
                                    ..Vec4F::default()
                                },
                                &mut test,
                            );
                        }
                        _ => {}
                    }

                    for t in tris_to_add.clone() {
                        list_triangles.push_back(t);
                    }
                }
                new_triangles = list_triangles.len();
            }

            for t in list_triangles {
                self.fill_triangle(
                    t.p[0].x as i32,
                    t.p[0].y as i32,
                    t.p[1].x as i32,
                    t.p[1].y as i32,
                    t.p[2].x as i32,
                    t.p[2].y as i32,
                    t.color,
                );
                self.draw_triangle(
                    t.p[0].x as i32, t.p[0].y as i32,
                    t.p[1].x as i32, t.p[1].y as i32,
                    t.p[2].x as i32, t.p[2].y as i32,
                    0
                );
            }
        }
        self.draw_string(
            10,
            100,
            format!("TRIANGLES: {}", triangles_to_raster.len()).as_str(),
            0xFFFFFF,
        );
    }

    fn handle_input(&mut self, elapsed_time: f32) {
        self.window.borrow().get_keys().iter().for_each(|key| {
            match key {
                Key::Space => {
                    self.camera.y += 8.0 * elapsed_time;
                    if self.window.borrow().is_key_down(Key::LeftCtrl) {
                        self.camera.y += 16.0 * elapsed_time;
                    }
                }
                Key::LeftShift => {
                    self.camera.y -= 8.0 * elapsed_time;
                }
                Key::W => {
                    self.camera += self.look_dir * (8.0 * elapsed_time);
                    if self.window.borrow().is_key_down(Key::LeftCtrl) {
                        self.camera += self.look_dir * (8.0 * elapsed_time) * 2.0;
                    }
                }
                Key::S => {
                    self.camera -= self.look_dir * (8.0 * elapsed_time);
                }
                Key::A => {
                    self.yaw -= 2.0 * elapsed_time;
                }
                Key::D => {
                    self.yaw += 2.0 * elapsed_time;
                }
                _ => {}
            }
        });
    }

    fn random_rainbow_color() -> u32 {
        let colors = vec![
            0xFF0000, 0xFFA500, 0xFFFF00, 0x008000, 0x0000FF, 0x4B0082, 0xEE82EE,
        ];
        let index = rand::thread_rng().gen_range(0..colors.len());
        colors[index]
    }

    fn get_color(lum: f32) -> u32 {
        let pixel_bw = (13.0 * lum) as i32;

        let color = match pixel_bw {
            0 => 0x000000,
            1 => 0x151515,
            2 => 0x2A2A2A,
            3 => 0x3F3F3F,
            4 => 0x555555,
            5 => 0x6A6A6A,
            6 => 0x7F7F7F,
            7 => 0x949494,
            8 => 0xAAAAAA,
            9 => 0xBFBFBF,
            10 => 0xD4D4D4,
            11 => 0xE9E9E9,
            12 => 0xFFFFFF,
            _ => 0x000000,
        };

        color
    }

    fn clip_against_plane<'a>(
        &mut self,
        plane_p: &'a mut Vec4F,
        plane_n: &mut Vec4F,
        in_tri: &'a mut Triangle,
    ) -> Vec<Triangle> {
        *plane_n = plane_n.normalize();

        let (mut out_tri1, mut out_tri2) = (Triangle::default(), Triangle::default());

        let dist = |p: &mut Vec4F| {
            let _n = p.normalize();
            return plane_n.x * p.x + plane_n.y * p.y + plane_n.z * p.z
                - plane_n.dot_product(&plane_p);
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
            out_tri1.p[1] =
                Vec4F::intersects_plane(plane_p, plane_n, &inside_points[0], &outside_points[0]);
            out_tri1.p[2] =
                Vec4F::intersects_plane(plane_p, plane_n, &inside_points[0], &outside_points[1]);

            return vec![out_tri1];
        }

        if inside_points_count == 2 && outside_points_count == 1 {
            out_tri1.color = in_tri.color;
            out_tri2.color = in_tri.color;

            out_tri1.p[0] = inside_points[0];
            out_tri1.p[1] = inside_points[1];
            out_tri1.p[2] =
                Vec4F::intersects_plane(plane_p, plane_n, &inside_points[0], &outside_points[0]);

            out_tri2.p[0] = inside_points[1];
            out_tri2.p[1] = out_tri1.p[2];
            out_tri2.p[2] =
                Vec4F::intersects_plane(plane_p, plane_n, &inside_points[1], &outside_points[0]);

            return vec![out_tri1, out_tri2];
        }

        vec![]
    }

    pub fn draw_square(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, cal: u32) {
        self.draw_triangle(x1, y1, x2, y1, x2, y2, cal);
        self.draw_triangle(x1, y1, x1, y2, x2, y2, cal);
    }

    pub fn draw(&mut self, x: i32, y: i32, col: u32) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.buffer[(y * self.width as i32 + x) as usize] = col;
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
            self.draw(xc + x, yc + y, 0xFFFFFF);
            self.draw(xc + y, yc + x, 0xFFFFFF);
            self.draw(xc - y, yc + x, 0xFFFFFF);
            self.draw(xc - x, yc + y, 0xFFFFFF);
            self.draw(xc - x, yc - y, 0xFFFFFF);
            self.draw(xc - y, yc - x, 0xFFFFFF);
            self.draw(xc + y, yc - x, 0xFFFFFF);
            self.draw(xc + x, yc - y, 0xFFFFFF);
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
        col: u32,
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

    pub fn fill_triangle(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        col: u32,
    ) {
        // count_calls(self);

        // Sort the points by y-coordinate
        let mut points = [(x1, y1), (x2, y2), (x3, y3)];
        points.sort_by_key(|p| p.1);

        let (x1, y1) = points[0];
        let (x2, y2) = points[1];
        let (x3, y3) = points[2];

        // Calculate the slopes
        let slope_a = if y2 - y1 != 0 {
            (x2 - x1) as f32 / (y2 - y1) as f32
        } else {
            0.0
        };
        let slope_b = if y3 - y1 != 0 {
            (x3 - x1) as f32 / (y3 - y1) as f32
        } else {
            0.0
        };
        let slope_c = if y3 - y2 != 0 {
            (x3 - x2) as f32 / (y3 - y2) as f32
        } else {
            0.0
        };

        // Draw the triangle
        for y in y1..=y2 {
            let xa = x1 as f32 + slope_a * (y - y1) as f32;
            let xb = x1 as f32 + slope_b * (y - y1) as f32;
            self.fill_line(xa.round() as i32, xb.round() as i32, y, col);
        }
        for y in y2..=y3 {
            let xa = x2 as f32 + slope_c * (y - y2) as f32;
            let xb = x1 as f32 + slope_b * (y - y1) as f32;
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
        if x >= self.width as i32 {
            x = self.width as i32 - 1;
        }
        if y < 0 {
            y = 0;
        }
        if y >= self.height as i32 {
            y = self.height as i32 - 1;
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
        let font: Font<'static> =
            Font::try_from_bytes(include_bytes!(r"assets\pixelfont.ttf") as &[u8]).unwrap();
        let height: f32 = 20f32; // adjust as needed
        let scale = Scale {
            x: height,
            y: height,
        };
        let v_metrics = font.v_metrics(scale);
        let offset = point(x as f32, v_metrics.ascent + y as f32);
        let iter = font.layout(string, scale, offset);

        for g in iter {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let v = v * 0xFF as f32;
                    let x = x + bb.min.x as u32;
                    let y = y + bb.min.y as u32;
                    if v > 150.0 {
                        self.draw(x as i32, y as i32, col);
                    }
                });
            }
        }
    }

    fn get(&self, x: i32, y: i32) -> u32 {
        self.buffer[y as usize * self.width + x as usize]
    }
}
