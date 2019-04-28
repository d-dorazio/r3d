use rand::Rng;

use geo::ray::Ray;
use geo::Vec3;

/// Enum over all the supported `Material`s. Each variant dictates how light
/// interacts(reflects, refracts, etc..) with them. They're mainly composed of
/// an `albedo` field which is the intrinsic color of the material.
#[derive(Debug, PartialEq, Clone)]
pub struct Material {
    pub albedo: [f64; 4],
    pub diffuse_color: Vec3,
    pub specular_exponent: f64,
    pub refraction_index: f64,
}

impl Material {
    /// The `Lambertian` material is a perfectly matte or diffuse surface which
    /// is modeled after the [Lambertian reflectance model][0].
    ///
    /// [0]: https://en.wikipedia.org/wiki/Lambertian_reflectance
    pub const fn lambertian(diffuse_color: Vec3) -> Self {
        Material {
            diffuse_color,
            specular_exponent: 0.0,
            refraction_index: 0.0,
            albedo: [0.0; 4],
        }
    }

    /// A metallic material that reflects light as it comes in. The `fuzziness`
    /// attribute is how much to perturbate each reflected ray. A low value of
    /// `fuzziness` makes it reflect more accurately because the reflected rays
    /// will change less. On the other hand, an high value will make it a bit
    /// opaque while still reflecting its surroundings.
    pub const fn metal(diffuse_color: Vec3, _fuzziness: f64) -> Self {
        Material {
            diffuse_color,
            specular_exponent: 0.0,
            refraction_index: 0.0,
            albedo: [0.0; 4],
        }
    }

    /// Clear materials like glass and diamond are of type Dielectric and are
    /// identified by a refracion index. For example, glass has a refraction
    /// index in [1.3, 1.7] while diamond is 2.4.
    pub const fn dielectric(refraction_index: f64) -> Self {
        Material {
            diffuse_color: Vec3::zero(),
            refraction_index,
            specular_exponent: 0.0,
            albedo: [0.0; 4],
        }
    }

    // A light material is a material that does not reflect rays, but always
    // emits the given light.
    // pub const fn light(emittance: Vec3) -> Self {
    //     Material::Light { emittance }
    // }
}

pub fn lambertian_bounce(intersection: Vec3, n: Vec3, rng: &mut impl Rng) -> Ray {
    Ray::new(intersection, n + Vec3::random_unit(rng))
}

pub fn metal_bounce(
    ray: &Ray,
    intersection: Vec3,
    n: Vec3,
    fuzziness: f64,
    rng: &mut impl Rng,
) -> Ray {
    Ray::new(
        intersection,
        Ray::new(ray.dir.normalized(), n).reflect() + Vec3::random_unit(rng) * fuzziness,
    )
}

pub fn dielectric_bounce(
    ray: &Ray,
    intersection: Vec3,
    n: Vec3,
    refraction_index: f64,
    rng: &mut impl Rng,
) -> Ray {
    let outward_normal;
    let ref_ix;
    let cos;

    if ray.dir.dot(&n) > 0.0 {
        outward_normal = -n;
        ref_ix = refraction_index;

        // cos = ref_ix * ray.dir.dot(&n) / ray.dir.norm();
        cos = (1.0 - ref_ix.powi(2) * (1.0 - (ray.dir.dot(&n) / ray.dir.norm()).powi(2))).sqrt();
    } else {
        outward_normal = n;
        ref_ix = 1.0 / refraction_index;
        cos = -ray.dir.dot(&n) / ray.dir.norm();
    }

    let dir = match Ray::new(ray.dir, outward_normal).refract(ref_ix) {
        Some(refracted) => {
            let reflect_prob = schlick(cos, ref_ix);

            if rng.gen::<f64>() < reflect_prob {
                Ray::new(ray.dir, n).reflect()
            } else {
                refracted
            }
        }
        None => Ray::new(ray.dir, n).reflect(),
    };

    Ray::new(intersection, dir)
}

/// Approximate the [Fresnel factor][1] that is the factor or refracted light
/// between different optical media using [Schlick equations].
///
/// [0]: https://en.wikipedia.org/wiki/Schlick's_approximation
/// [1]: https://en.wikipedia.org/wiki/Fresnel_equations
fn schlick(cos: f64, refraction_index: f64) -> f64 {
    let r0 = (1.0 - refraction_index) / (1.0 + refraction_index).powi(2);

    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}
