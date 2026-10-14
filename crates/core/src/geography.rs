use crate::model::Place;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Geometry {
    pub kilometers: f64,
    pub central_angle: f64,
    pub delta_lat: f64,
    pub delta_lon: f64,
    pub shortest_lon: f64,
}
pub fn between(a: &Place, b: &Place) -> Geometry {
    let lat = (b.latitude - a.latitude).to_radians();
    let lon = (b.longitude - a.longitude).to_radians();
    let h = (lat / 2.).sin().powi(2)
        + a.latitude.to_radians().cos() * b.latitude.to_radians().cos() * (lon / 2.).sin().powi(2);
    let angle = 2. * h.clamp(0., 1.).sqrt().asin();
    let delta_lon = b.longitude - a.longitude;
    Geometry {
        kilometers: 6371.0088 * angle,
        central_angle: angle.to_degrees(),
        delta_lat: b.latitude - a.latitude,
        delta_lon,
        shortest_lon: ((delta_lon + 180.).rem_euclid(360.) - 180.).abs(),
    }
}

/// Sample the same spherical great circle used by `between`, in [longitude, latitude].
/// Exactly antipodal points have many shortest routes; choose a stable perpendicular plane.
pub fn route(a: &Place, b: &Place) -> Vec<[f64; 2]> {
    let vector = |p: &Place| {
        let (lat, lon) = (p.latitude.to_radians(), p.longitude.to_radians());
        [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
    };
    let start = vector(a);
    let end = vector(b);
    let dot = start
        .iter()
        .zip(end)
        .map(|(x, y)| x * y)
        .sum::<f64>()
        .clamp(-1., 1.);
    let mut tangent = std::array::from_fn::<_, 3, _>(|i| end[i] - dot * start[i]);
    let norm = tangent.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm < 1e-12 && dot > 0. {
        return vec![[a.longitude, a.latitude], [b.longitude, b.latitude]];
    }
    let angle = norm.atan2(dot);
    if norm < 1e-12 {
        let axis = (0..3)
            .min_by(|&i, &j| start[i].abs().total_cmp(&start[j].abs()))
            .unwrap();
        tangent =
            std::array::from_fn(|i| (if i == axis { 1. } else { 0. }) - start[axis] * start[i]);
    }
    let norm = tangent.iter().map(|x| x * x).sum::<f64>().sqrt();
    tangent.iter_mut().for_each(|x| *x /= norm);
    let steps = angle.to_degrees().ceil().max(1.) as usize;
    let mut points = (0..=steps)
        .map(|i| {
            let t = angle * i as f64 / steps as f64;
            let p: [f64; 3] = std::array::from_fn(|j| start[j] * t.cos() + tangent[j] * t.sin());
            [
                p[1].atan2(p[0]).to_degrees(),
                p[2].atan2(p[0].hypot(p[1])).to_degrees(),
            ]
        })
        .collect::<Vec<_>>();
    points[0] = [a.longitude, a.latitude];
    *points.last_mut().unwrap() = [b.longitude, b.latitude];
    points
}
