use geo::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::{render, Environment, Light, RenderConfig, Scene};

fn main() {
    let target = Vec3::new(0.0, 0.0, -1.0);
    let camera = Camera::look_at(Vec3::zero(), target, Vec3::new(0.0, 1.0, 0.0), 90.0);

    let scene = Scene::new(
        vec![
            Sphere::new(
                Vec3::new(0.0, 0.0, -1.0),
                0.5,
                Material::lambertian(Vec3::new(0.8, 0.3, 0.3)),
            ),
            Sphere::new(
                Vec3::new(0.0, 0.0, 100.0),
                200.0,
                Material::lambertian(Vec3::new(0.8, 0.1, 0.1)),
            ),
        ],
        vec![
            Light {
                intensity: 0.5,
                position: Vec3::new(3.0, 0.0, 0.0),
            },
            Light {
                intensity: 0.1,
                position: Vec3::new(-3.0, 0.0, 3.0),
            },
        ],
        Environment::Color(Vec3::new(0.1, 0.1, 0.1)),
    );

    let mut rng = rand::thread_rng();

    let img = render(
        &camera,
        &scene,
        &mut rng,
        &RenderConfig {
            width: 400,
            height: 200,
            samples: 5,
            max_bounces: 5,
        },
    );
    img.save("lights.png").unwrap();
}
