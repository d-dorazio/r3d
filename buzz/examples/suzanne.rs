use std::io;
use std::io::{BufReader, Cursor};

use geo::mesh::stl;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::facet::Facet;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::Object;
use buzz::{parallel_render, Environment, Light, RenderConfig, Scene};

// const MESH_MATERIAL: Material = Material::lambertian(Vec3::new(0.8, 0.1, 0.1));
const MESH_MATERIAL: Material = Material {
    albedo: [0.6, 0.3, 0.1, 0.0],
    diffuse_color: Vec3::new(0.4, 0.4, 0.3),
    refraction_index: 1.0,
    specular_exponent: 10.0,
};

pub fn main() -> io::Result<()> {
    let camera = Camera::look_at(
        Vec3::new(2.7, -2.7, 2.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        35.0,
    );

    let mut scene: Vec<Box<dyn Object>> = vec![
        // Box::new(Sphere::new(
        //     Vec3::new(-1.5, -4.5, 0.3),
        //     1.0,
        //     Material::light(Vec3::new(9.0, 9.0, 9.0)),
        // )),
        // Box::new(Sphere::new(
        //     Vec3::new(2.1, 4.5, 0.3),
        //     1.0,
        //     Material::light(Vec3::new(9.0, 9.0, 9.0)),
        // )),
    ];

    let (_, tris) = stl::load_binary_stl(BufReader::new(Cursor::new(
        &include_bytes!("../../data/suzanne.stl")[..],
    )))?;

    for t in tris {
        let t = t?;
        scene.push(Box::new(Facet::new(t, &MESH_MATERIAL)));
    }

    let environment = Environment::Color(Vec3::new(0.2, 0.3, 0.36));
    let img = parallel_render(
        &camera,
        &Scene::new(
            scene,
            vec![
                Light {
                    position: Vec3::new(-1.5, -4.5, 0.3),
                    intensity: 1.5,
                },
                Light {
                    position: Vec3::new(2.1, 4.5, 0.3),
                    intensity: 1.5,
                },
            ],
            environment,
        ),
        &RenderConfig {
            width: 1920 / 2,
            height: 1080 / 2,
            max_bounces: 4,
            samples: 1,
        },
    );

    img.save("suzanne.png")
}
