#![allow(clippy::useless_let_if_seq)]

pub mod camera;
pub mod facet;
pub mod material;
pub mod sphere;

use std::ops::Deref;

use image::{Rgb, RgbImage};
use rand::Rng;
use rayon::prelude::*;

use geo::ray::Ray;
use geo::spatial_index::bvh::Bvh;
use geo::spatial_index::Shape;
use geo::{vec3, Vec3};

use camera::Camera;
use material::{dielectric_bounce, lambertian_bounce, metal_bounce, Material};

/// An `Object` that can be rendered.
pub trait Object: Shape + Sync {
    /// Getter for the `Material` the `Object` is made of.
    fn material(&self) -> &Material;

    /// Calculate the normal for the given point `p`. This method should never
    /// be called if the `Object` does not intersect it.
    fn normal_at(&self, p: Vec3) -> Vec3;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Light {
    pub position: Vec3,
    pub intensity: f64,
}

/// A `Scene` is a collection of objects that can be rendered.
#[derive(Debug)]
pub struct Scene<O: Object> {
    objects: Bvh<O>,
    environment: Environment,
    lights: Vec<Light>,
}

/// The `Environment` surrounding the objects in a `Scene`. All the rays that
/// don't hit any objects will hit the environment.
#[derive(Debug, PartialEq, Clone)]
pub enum Environment {
    /// The `Environment` is a simple RGB color where each channel is in [0, 1].
    Color(Vec3),

    /// The `Environment` is a simple linear gradient between two RGB colors.
    LinearGradient(Vec3, Vec3),
}

impl<O: Object> Scene<O> {
    /// Create a new `Scene` with the given objects inside the given
    /// `Environment`.
    pub fn new(
        objects: impl IntoIterator<Item = O>,
        lights: Vec<Light>,
        environment: Environment,
    ) -> Self {
        Scene {
            objects: objects.into_iter().collect(),
            environment,
            lights,
        }
    }

    /// Calculate the intersection between a `Ray` and all the objects in the
    /// scene returning the closest object (along with its intersection result)
    /// to the ray.
    pub fn intersection<'s>(&'s self, ray: &'s Ray) -> Option<(&'s O, f64)> {
        self.objects
            .intersections(ray)
            .min_by(|(_, t0), (_, t1)| t0.partial_cmp(t1).unwrap())
    }
}

/// Simple struct to hold rendering params together.
#[derive(Debug, PartialEq, Clone)]
pub struct RenderConfig {
    /// how many samples to take for each pixel to find out its color. This can
    /// help reduce aliasing since the final color is the average of all the
    /// samples.
    pub samples: u32,

    /// the maximum number of bounces a ray can do. An higher value will give
    /// better results in scenes with a lot of reflective objects.
    pub max_bounces: u32,

    /// width and height of the rendered image.
    pub width: u32,
    pub height: u32,
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions.
pub fn render(
    camera: &Camera,
    scene: &Scene<impl Object>,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> image::RgbImage {
    let mut img = RgbImage::new(config.width, config.height);

    for (x, y, pix) in img.enumerate_pixels_mut() {
        *pix = render_pixel((x, y), camera, scene, rng, config);
    }

    img
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions concurrently.
pub fn parallel_render(
    camera: &Camera,
    scene: &Scene<impl Object>,
    config: &RenderConfig,
) -> image::RgbImage {
    let mut img = RgbImage::new(config.width, config.height);

    img.par_chunks_mut(3)
        .zip((0_u32..config.width * config.height).into_par_iter())
        .for_each(|(pix, i)| {
            let x = i % config.width;
            let y = i / config.width;

            let rgb = render_pixel(
                (x as u32, y as u32),
                camera,
                scene,
                &mut rand::thread_rng(),
                config,
            );

            pix[0] = rgb.data[0];
            pix[1] = rgb.data[1];
            pix[2] = rgb.data[2];
        });

    img
}

/// Render a single pixel of an image from a `Scene` and `Camera`.
pub fn render_pixel(
    (x, y): (u32, u32),
    camera: &Camera,
    scene: &Scene<impl Object>,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Rgb<u8> {
    let mut c = (0..config.samples)
        .map(|_| {
            let r = camera.cast_ray((x, y), (config.width, config.height), rng);
            sample(&scene, &r, 0, rng, config)
        })
        .sum::<Vec3>()
        / f64::from(config.samples);

    // gamma correct pixels
    c.x = c.x.sqrt();
    c.y = c.y.sqrt();
    c.z = c.z.sqrt();

    // dbg!((x,y));

    Rgb {
        data: [
            (c.x * 255.0) as u8,
            (c.y * 255.0) as u8,
            (c.z * 255.0) as u8,
        ],
    }
}

fn sample(
    scene: &Scene<impl Object>,
    ray: &Ray,
    depth: u32,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Vec3 {
    match scene.intersection(ray) {
        None => sample_environment(scene, ray),

        Some(_) if depth >= config.max_bounces => Vec3::zero(),
        Some((s, t)) => {
            let material = s.material();

            let intersection = ray.point_at(t);
            let n = s.normal_at(intersection);

            let mut diffuse_light_intensity = 0.0;
            let mut specular_light_intensity = 0.0;

            for light in &scene.lights {
                let mut light_dir = light.position - intersection;
                let light_dist2 = light_dir.norm2();
                light_dir.normalize();

                let light_ray = Ray::new(intersection, light_dir);

                // check if point is on the shadow
                if let Some((_, t)) = scene.intersection(&light_ray) {
                    if light_ray.point_at(t).dist2(&light_ray.origin) < light_dist2 {
                        continue;
                    }
                }

                diffuse_light_intensity += light_dir.dot(&n).max(0.0) * light.intensity;
                specular_light_intensity += (-Ray::new(-light_dir, n).reflect())
                    .dot(&ray.dir)
                    .max(0.0)
                    .powf(material.specular_exponent)
                    * light.intensity;
            }

            let reflect_ray = Ray::new(intersection, Ray::new(ray.dir.normalized(), n).reflect());
            let reflected_color = sample(scene, &reflect_ray, depth + 1, rng, config);

            let mut c = material.diffuse_color * diffuse_light_intensity * material.albedo[0];
            c += Vec3::new(1.0, 1.0, 1.0) * specular_light_intensity * material.albedo[1];
            c += reflected_color * material.albedo[2];

            if let Some(d) = Ray::new(ray.dir.normalized(), n).refract(material.refraction_index) {
                let refract_ray = Ray::new(intersection, d);
                let refracted_color = sample(scene, &refract_ray, depth + 1, rng, config);

                c += refracted_color * material.albedo[3];
            }
            c
        }
    }
}

fn sample_environment(scene: &Scene<impl Object>, ray: &Ray) -> Vec3 {
    match scene.environment {
        Environment::Color(c) => c,
        Environment::LinearGradient(a, b) => {
            let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);
            vec3::lerp(a, b, t)
        }
    }
}

impl<T> Object for Box<T>
where
    T: Object + ?Sized,
{
    fn material(&self) -> &Material {
        self.deref().material()
    }

    fn normal_at(&self, p: Vec3) -> Vec3 {
        self.deref().normal_at(p)
    }
}
