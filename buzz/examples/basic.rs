use geo::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::{render, Environment, RenderConfig, Scene};

pub fn main() {
    let target = Vec3::new(0.0, 0.0, -1.0);
    let camera = Camera::look_at(Vec3::zero(), target, Vec3::new(0.0, 1.0, 0.0), 90.0)
        .with_focus(target, 0.25);

    let scene = Scene::new(
        vec![
            Sphere::new(
                Vec3::new(0.0, 0.0, -1.0),
                0.5,
                Material::lambertian(Vec3::new(0.8, 0.3, 0.3)),
            ),
            Sphere::new(
                Vec3::new(0.0, -100.5, -1.0),
                100.0,
                Material::lambertian(Vec3::new(0.8, 0.8, 0.0)),
            ),
            Sphere::new(
                Vec3::new(1.0, 0.0, -1.0),
                0.5,
                Material::metal(Vec3::new(0.8, 0.6, 0.2), 0.3),
            ),
            Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.5, Material::dielectric(1.5)),
        ],
        vec![],
        Environment::Color(Vec3::new(0.2, 0.2, 0.8)),
    );

    let mut rng = rand::thread_rng();

    let img = render(
        &camera,
        &scene,
        &mut rng,
        &RenderConfig {
            width: 400,
            height: 200,
            samples: 1,
            max_bounces: 5,
        },
    );
    img.save("debug.ppm").unwrap();
}
