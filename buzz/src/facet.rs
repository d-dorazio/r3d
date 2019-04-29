use geo::mesh::stl;
use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::triangle;
use geo::{Aabb, Vec3};

use crate::material::Material;
use crate::Object;

#[derive(Debug, PartialEq, Clone)]
pub struct Facet<'a> {
    positions: [Vec3; 3],
    normal: Vec3,
    material: &'a Material,
}

impl<'a> Facet<'a> {
    pub fn new(tri: stl::StlTriangle, material: &'a Material) -> Self {
        let normal = geo::triangle::normal(&tri.positions[0], &tri.positions[1], &tri.positions[2]);

        Facet {
            positions: tri.positions,
            normal,
            material,
        }
    }
}

impl<'a> Shape for Facet<'a> {
    fn bbox(&self) -> Aabb {
        Aabb::new(self.positions[0])
            .expanded(&self.positions[1])
            .expanded(&self.positions[2])
    }

    fn intersection(&self, ray: &Ray) -> Option<f64> {
        triangle::ray_intersection(
            (self.positions[0], self.positions[1], self.positions[2]),
            ray,
        )
    }
}

impl<'a> Object for Facet<'a> {
    fn material(&self) -> &Material {
        &self.material
    }

    fn normal_at(&self, pt: Vec3) -> Vec3 {
        self.normal

        // let bary = triangle::barycentric(
        //     (&self.positions[0], &self.positions[1], &self.positions[2]),
        //     &pt,
        // )
        // .expect("point outside triangle");

        // let mut n = self.positions[0] * bary.x;
        // n += self.positions[1] * bary.y;
        // n += self.positions[2] * bary.z;
        // n.normalize();

        // n
    }
}
