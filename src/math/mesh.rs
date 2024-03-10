use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use crate::drawer::{Triangle, Vec4F};

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub tris: Vec<Triangle>
}

impl Default for Mesh {
    fn default() -> Self {
        Mesh { tris: Vec::new() }
    }
}

impl Mesh {
    pub fn parse_obj_file(&mut self, filename: &str) -> Self {
        let path = Path::new(filename);
        let file = File::open(path).expect("Cannot open file");
        let mut lines = io::BufReader::new(file).lines();

        let mut major_ver: u32 = 0;

        if let Some(Ok(line)) = lines.next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let ver = parts[2];
            let major_ver_str: Vec<&str> = if ver.starts_with('v') {
                ver[1..].split('.').collect()
            } else {
                ver.split('.').collect()
            };
            major_ver = major_ver_str[0].parse::<u32>().unwrap();
        }

        println!("{}", major_ver);

        match major_ver {
            2 => self.extract_model_from_obj2(filename),
            4 => self.extract_model_from_obj4(filename),
            _ => panic!("Cannot parse obj file: {}", filename),
        }
    }

    pub fn extract_model_from_obj4(&mut self, filename: &str) -> Self {
        let path = Path::new(filename);
        let file = File::open(path).expect("Cannot open file");
        let reader = io::BufReader::new(file);

        let mut vertices: Vec<Vec4F> = Vec::new();
        for line in reader.lines().map(|line| line.unwrap()) {
            let mut parts = line.split_whitespace();
            if let Some(first) = parts.next() {
                if first == "v" {
                    let x = parts.next().unwrap().parse::<f32>().unwrap();
                    let y = parts.next().unwrap().parse::<f32>().unwrap();
                    let z = parts.next().unwrap().parse::<f32>().unwrap();

                    let vec = Vec4F { x, y, z, ..Vec4F::default() };
                    vertices.push(vec);
                }
                else if first == "f" {
                    let first_numbers: Vec<i32> = parts.map(|part| part.split('/').next().unwrap().parse::<i32>().unwrap())
                        .collect();


                    let tri = Triangle {
                        p: [
                            vertices[first_numbers[0] as usize - 1],
                            vertices[first_numbers[1] as usize - 1],
                            vertices[first_numbers[2] as usize - 1],
                        ],
                        color: 0xFFFFFF
                    };

                    self.tris.push(tri);
                }
            }
        }

        self.clone()
    }

    pub fn extract_model_from_obj2(&mut self, filename: &str) -> Self {
        let path = Path::new(filename);
        let file = File::open(path).expect("Cannot open file");
        let reader = io::BufReader::new(file);

        let mut vertices: Vec<Vec4F> = Vec::new();
        for line in reader.lines().map(|line| line.unwrap()) {
            let mut parts = line.split_whitespace();
            if let Some(first) = parts.next() {
                if first == "v" {
                    let x = parts.next().unwrap().parse::<f32>().unwrap();
                    let y = parts.next().unwrap().parse::<f32>().unwrap();
                    let z = parts.next().unwrap().parse::<f32>().unwrap();

                    let vec = Vec4F { x, y, z, ..Vec4F::default()};
                    vertices.push(vec);
                }
                else if first == "f" {
                    let a = parts.next().unwrap().parse::<i32>().unwrap();
                    let b = parts.next().unwrap().parse::<i32>().unwrap();
                    let c = parts.next().unwrap().parse::<i32>().unwrap();

                    let tri = Triangle {
                        p: [
                            vertices[a as usize - 1],
                            vertices[b as usize - 1],
                            vertices[c as usize - 1],
                        ],
                        color: 0xFFFFFF
                    };

                    self.tris.push(tri);
                }
            }
        }

        self.clone()
    }
}