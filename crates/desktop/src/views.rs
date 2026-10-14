use atlas_core::{
    analysis::Feature,
    model::*,
    sky::{self, BodyPosition, SkySnapshot},
};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::sync::OnceLock;

#[derive(Default)]
pub struct MapView {
    pub globe: bool,
    pub zoom: f32,
    pub pan: Vec2,
    pub longitude: f64,
    pub latitude: f64,
    pub show_labels: bool,
    pub context_places: Vec<String>,
    pub visible_places: Option<std::collections::HashSet<String>>,
    pub comparison: [Option<String>; 2],
    pub click_target: u8,
    pub show_connection: bool,
}
pub enum MapAction {
    Observer(String),
    NewPoint(f64, f64),
    Remove(String),
}
struct Land {
    points: Vec<[f64; 2]>,
    indices: Vec<usize>,
}
fn land() -> &'static Vec<Land> {
    static LAND: OnceLock<Vec<Land>> = OnceLock::new();
    LAND.get_or_init(|| {
        let json: serde_json::Value = serde_json::from_str(include_str!("../assets/land.geojson"))
            .expect("Bundled Natural Earth JSON");
        let mut out = vec![];
        for f in json["features"].as_array().unwrap() {
            let geometry = &f["geometry"];
            let polys = if geometry["type"] == "Polygon" {
                vec![&geometry["coordinates"]]
            } else {
                geometry["coordinates"].as_array().unwrap().iter().collect()
            };
            for polygon in polys {
                let mut points = vec![];
                let mut holes = vec![];
                for (i, ring) in polygon.as_array().unwrap().iter().enumerate() {
                    if i > 0 {
                        holes.push(points.len());
                    }
                    for point in ring.as_array().unwrap() {
                        points.push([point[0].as_f64().unwrap(), point[1].as_f64().unwrap()]);
                    }
                }
                let data = points.iter().flat_map(|p| *p).collect::<Vec<_>>();
                let indices =
                    earcutr::earcut(&data, &holes, 2).expect("Natural Earth polygon triangulation");
                out.push(Land { points, indices });
            }
        }
        out
    })
}
pub fn globe_point(lon: f64, lat: f64, center_lon: f64, center_lat: f64) -> (f32, f32, f32) {
    let lat = lat.to_radians();
    let mid = center_lat.to_radians();
    let dl = (lon - center_lon).to_radians();
    (
        (lat.cos() * dl.sin()) as f32,
        (mid.cos() * lat.sin() - mid.sin() * lat.cos() * dl.cos()) as f32,
        (mid.sin() * lat.sin() + mid.cos() * lat.cos() * dl.cos()) as f32,
    )
}
fn center_globe(view: &mut MapView, points: &[&Place]) {
    let (x, y, z) = points.iter().fold((0., 0., 0.), |(x, y, z), p| {
        let (lat, lon) = (p.latitude.to_radians(), p.longitude.to_radians());
        (
            x + lat.cos() * lon.cos(),
            y + lat.cos() * lon.sin(),
            z + lat.sin(),
        )
    });
    if x.hypot(y).hypot(z) > 1e-9 {
        view.longitude = y.atan2(x).to_degrees();
        view.latitude = z.atan2(x.hypot(y)).to_degrees();
    }
}
pub fn draw_earth(
    ui: &mut egui::Ui,
    view: &mut MapView,
    d: &Dataset,
    s: &Session,
    features: &[Feature],
    snapshot: Option<&SkySnapshot>,
) -> Option<MapAction> {
    let mut action = None;
    let pair = view
        .comparison
        .iter()
        .filter_map(|id| d.places.iter().find(|p| Some(&p.id) == id.as_ref()))
        .collect::<Vec<_>>();
    let map_width = ui.available_width().min(1050.);
    if view.zoom == 0. {
        view.zoom = 1.;
        view.show_labels = true;
        view.show_connection = true;
        if view.globe {
            center_globe(view, &pair);
        }
    }
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut view.globe, false, "World map");
        if ui
            .selectable_value(&mut view.globe, true, "Globe")
            .changed()
        {
            center_globe(view, &pair);
        }
        ui.checkbox(&mut view.show_labels, "Place labels");
        ui.checkbox(&mut view.show_connection, "Connect A/B");
        if ui.button("Reset view").clicked() {
            view.zoom = 1.;
            view.pan = Vec2::ZERO;
            view.longitude = 0.;
            view.latitude = 0.;
        }
        if ui.button("Center observer").clicked() {
            if let Some(p) = d
                .places
                .iter()
                .find(|p| Some(&p.id) == s.active_place.as_ref())
            {
                view.longitude = p.longitude;
                view.latitude = p.latitude;
                view.pan = egui::vec2(
                    -(p.longitude / 360. * map_width as f64) as f32 * view.zoom,
                    (p.latitude / 360. * map_width as f64) as f32 * view.zoom,
                );
            }
        }
        let points = d
            .places
            .iter()
            .filter(|p| {
                view.visible_places
                    .as_ref()
                    .is_none_or(|ids| ids.contains(&p.id))
            })
            .collect::<Vec<_>>();
        if ui
            .add_enabled(!points.is_empty(), egui::Button::new("Fit locations"))
            .clicked()
        {
            let min_lon = points
                .iter()
                .map(|p| p.longitude)
                .fold(f64::INFINITY, f64::min);
            let max_lon = points
                .iter()
                .map(|p| p.longitude)
                .fold(f64::NEG_INFINITY, f64::max);
            let min_lat = points
                .iter()
                .map(|p| p.latitude)
                .fold(f64::INFINITY, f64::min);
            let max_lat = points
                .iter()
                .map(|p| p.latitude)
                .fold(f64::NEG_INFINITY, f64::max);
            view.longitude = (min_lon + max_lon) / 2.;
            view.latitude = (min_lat + max_lat) / 2.;
            if view.globe {
                view.zoom = 1.;
                center_globe(view, &points);
            } else {
                view.zoom = ((360. / (max_lon - min_lon).max(10.)) as f32 * 0.8)
                    .min((180. / (max_lat - min_lat).max(10.)) as f32 * 0.8)
                    .clamp(1., 12.);
                view.pan = egui::vec2(
                    -(view.longitude / 360. * map_width as f64) as f32 * view.zoom,
                    (view.latitude / 360. * map_width as f64) as f32 * view.zoom,
                );
            }
        }
    });
    ui.weak("Drag to pan/rotate. Scroll to zoom. Click a marker to select it; right-click to remove it. Double-click empty world-map space to add a place. Undo restores removals.");
    let width = ui.available_width().min(1050.);
    let height = if view.globe {
        width.min(350.)
    } else {
        width / 2.
    };
    let (response, base_painter) =
        ui.allocate_painter(Vec2::new(width, height), egui::Sense::click_and_drag());
    let r = response.rect;
    let painter = base_painter.with_clip_rect(r);
    if response.dragged() {
        let delta = response.drag_delta();
        if view.globe {
            view.longitude = sky::wrap_signed(view.longitude - delta.x as f64 * 0.4);
            view.latitude = (view.latitude + delta.y as f64 * 0.3).clamp(-89., 89.);
        } else {
            view.pan += delta;
        }
    }
    if response.hovered() {
        let scroll = ui.ctx().input_mut(|i| {
            let v = i.smooth_scroll_delta.y;
            i.smooth_scroll_delta = Vec2::ZERO;
            v
        });
        if scroll != 0. {
            let factor = (scroll * 0.003).exp();
            let old = view.zoom;
            view.zoom = (old * factor).clamp(1., 12.);
            if !view.globe {
                if let Some(pos) = response.hover_pos() {
                    view.pan = pos - r.center() - (pos - r.center() - view.pan) * (view.zoom / old);
                }
            }
        }
    }
    painter.rect_filled(r, 6., Color32::from_rgb(12, 30, 45));
    let radius = (height.min(width) * 0.45) * if view.globe { view.zoom.min(2.) } else { 1. };
    if view.globe {
        painter.circle_filled(r.center(), radius, Color32::from_rgb(22, 49, 67));
    }
    let project = |lon: f64, lat: f64| -> Option<Pos2> {
        if view.globe {
            let (x, y, z) = globe_point(lon, lat, view.longitude, view.latitude);
            (z >= 0.).then(|| r.center() + egui::vec2(x, -y) * radius)
        } else {
            Some(
                r.center()
                    + view.pan
                    + egui::vec2(
                        (lon / 360. * width as f64) as f32,
                        (-lat / 180. * height as f64) as f32,
                    ) * view.zoom,
            )
        }
    };
    for shape in land() {
        if !view.globe {
            let mut mesh = egui::Mesh::default();
            for p in &shape.points {
                mesh.colored_vertex(project(p[0], p[1]).unwrap(), Color32::from_rgb(45, 79, 81));
            }
            mesh.indices = shape.indices.iter().map(|i| *i as u32).collect();
            painter.add(egui::Shape::mesh(mesh));
        }
        for pair in shape.points.windows(2) {
            if (pair[0][0] - pair[1][0]).abs() > 180. {
                continue;
            }
            if let (Some(a), Some(b)) = (
                project(pair[0][0], pair[0][1]),
                project(pair[1][0], pair[1][1]),
            ) {
                painter.line_segment([a, b], Stroke::new(1., Color32::from_rgb(111, 155, 139)));
            }
        }
    }
    if let Some(sky) = snapshot {
        let mut mesh = egui::Mesh::default();
        for lat in (-90..90).step_by(5) {
            for lon in (-180..180).step_by(5) {
                let coords = [
                    (lon as f64, lat as f64),
                    ((lon + 5) as f64, lat as f64),
                    ((lon + 5) as f64, (lat + 5) as f64),
                    (lon as f64, (lat + 5) as f64),
                ];
                if let Some(points) = coords
                    .iter()
                    .map(|(lon, lat)| project(*lon, *lat))
                    .collect::<Option<Vec<_>>>()
                {
                    let index = mesh.vertices.len() as u32;
                    for (pos, (lon, lat)) in points.into_iter().zip(coords) {
                        let alt = sky::solar_altitude(lat, lon, sky);
                        let alpha = ((-alt / 12.).clamp(0., 1.) * 175.) as u8;
                        mesh.colored_vertex(pos, Color32::from_black_alpha(alpha));
                    }
                    mesh.add_triangle(index, index + 1, index + 2);
                    mesh.add_triangle(index, index + 2, index + 3);
                }
            }
        }
        painter.add(egui::Shape::mesh(mesh));
    }
    // Latitude/longitude grid on both projections.
    for lat in (-60..=60).step_by(30) {
        for lon in (-180..180).step_by(3) {
            if let (Some(a), Some(b)) = (
                project(lon as f64, lat as f64),
                project((lon + 3) as f64, lat as f64),
            ) {
                painter.line_segment(
                    [a, b],
                    Stroke::new(0.6, Color32::from_rgba_unmultiplied(145, 185, 203, 95)),
                );
            }
        }
    }
    for lon in (-180..180).step_by(30) {
        for lat in (-90..90).step_by(3) {
            if let (Some(a), Some(b)) = (
                project(lon as f64, lat as f64),
                project(lon as f64, (lat + 3) as f64),
            ) {
                painter.line_segment(
                    [a, b],
                    Stroke::new(0.6, Color32::from_rgba_unmultiplied(145, 185, 203, 95)),
                );
            }
        }
    }
    if !view.globe {
        for lon in (-180..=180).step_by(60) {
            if let Some(p) = project(lon as f64, 0.) {
                painter.text(
                    p,
                    egui::Align2::CENTER_CENTER,
                    format!("{lon}°"),
                    egui::FontId::proportional(11.),
                    Color32::LIGHT_GRAY,
                );
            }
        }
        for lat in (-60..=60).step_by(30) {
            if let Some(p) = project(-177., lat as f64) {
                painter.text(
                    p,
                    egui::Align2::LEFT_CENTER,
                    format!("{lat}°"),
                    egui::FontId::proportional(11.),
                    Color32::LIGHT_GRAY,
                );
            }
        }
    }
    if view.show_connection {
        if let (Some(a), Some(b)) = (
            d.places
                .iter()
                .find(|p| Some(&p.id) == view.comparison[0].as_ref()),
            d.places
                .iter()
                .find(|p| Some(&p.id) == view.comparison[1].as_ref()),
        ) {
            for segment in atlas_core::geography::route(a, b).windows(2) {
                if !view.globe && (segment[0][0] - segment[1][0]).abs() > 180. {
                    continue;
                }
                if let (Some(a), Some(b)) = (
                    project(segment[0][0], segment[0][1]),
                    project(segment[1][0], segment[1][1]),
                ) {
                    painter.line_segment([a, b], Stroke::new(2.5, Color32::from_rgb(255, 191, 99)));
                }
            }
        }
    }
    let mut nearby = vec![];
    for place in &d.places {
        if view
            .visible_places
            .as_ref()
            .is_some_and(|ids| !ids.contains(&place.id))
            && Some(&place.id) != s.active_place.as_ref()
        {
            continue;
        }
        if let Some(pos) = project(place.longitude, place.latitude) {
            let active = Some(&place.id) == s.active_place.as_ref();
            let slot = view
                .comparison
                .iter()
                .position(|id| id.as_ref() == Some(&place.id));
            let hit = features.iter().any(|f| {
                f.owner_id == place.id && f.owner == place.name && f.matches(&s.query, &s.settings)
            });
            let color = if hit {
                super::app::HIT
            } else if active {
                Color32::WHITE
            } else if let Some(slot) = slot {
                if slot == 0 {
                    super::app::ACCENT
                } else {
                    Color32::from_rgb(255, 191, 99)
                }
            } else {
                super::app::ACCENT
            };
            painter.circle_filled(pos, if active || slot.is_some() { 6. } else { 4. }, color);
            if active {
                painter.circle_stroke(pos, 10., Stroke::new(1.5, color));
            }
            if view.show_labels {
                painter.text(
                    pos + egui::vec2(9., -10.),
                    egui::Align2::LEFT_BOTTOM,
                    slot.map_or_else(
                        || place.name.clone(),
                        |i| format!("{} · {}", if i == 0 { "A" } else { "B" }, place.name),
                    ),
                    egui::FontId::proportional(12.),
                    color,
                );
            }
            if response.hover_pos().is_some_and(|p| p.distance(pos) < 12.) {
                response.clone().on_hover_text(format!(
                    "{}\nLatitude {}°, longitude {}°\n{}",
                    place.name, place.latitude, place.longitude, place.context
                ));
            }
            if let Some(pointer) = response
                .interact_pointer_pos()
                .or_else(|| response.hover_pos())
            {
                if r.contains(pos) && pointer.distance(pos) < 12. {
                    nearby.push((pointer.distance(pos), place));
                }
            }
        }
    }
    nearby.sort_by(|(a, _), (b, _)| a.total_cmp(b));
    if response.clicked() {
        if let Some((_, place)) = nearby.first() {
            action = Some(MapAction::Observer(place.id.clone()));
        }
    }
    if response.secondary_clicked() {
        view.context_places = nearby.iter().map(|(_, p)| p.id.clone()).collect();
    }
    response.context_menu(|ui| {
        let places = view
            .context_places
            .iter()
            .filter_map(|id| d.places.iter().find(|p| p.id == *id))
            .collect::<Vec<_>>();
        if places.is_empty() {
            ui.label("Right-click a saved marker to remove it.");
        }
        for place in places {
            ui.strong(&place.name);
            ui.weak(format!("{}, {}", place.latitude, place.longitude));
            if ui.button(format!("Remove {}", place.name)).clicked() {
                action = Some(MapAction::Remove(place.id.clone()));
                ui.close();
            }
        }
    });
    if let Some(sky) = snapshot {
        if let Some(p) = project(sky.subsolar_longitude, sky.subsolar_latitude) {
            painter.circle_filled(p, 5., Color32::YELLOW);
            painter.text(
                p + egui::vec2(8., 8.),
                egui::Align2::LEFT_TOP,
                "Sun overhead",
                egui::FontId::proportional(11.),
                Color32::YELLOW,
            );
        }
    }
    if view.globe {
        painter.circle_stroke(
            r.center(),
            radius,
            Stroke::new(1.5, Color32::from_gray(150)),
        );
    } else if let Some(pos) = response.hover_pos() {
        let local = (pos - r.center() - view.pan) / view.zoom;
        let lon = sky::wrap_signed(local.x as f64 / width as f64 * 360.);
        let lat = -local.y as f64 / height as f64 * 180.;
        if (-90. ..=90.).contains(&lat) {
            ui.weak(format!("Cursor: latitude {lat:.4}°, longitude {lon:.4}°"));
            if response.double_clicked() && nearby.is_empty() {
                action = Some(MapAction::NewPoint(lat, lon));
            }
        }
    }
    action
}
pub fn body_color(name: &str) -> Color32 {
    match name {
        "Sun" => Color32::from_rgb(255, 209, 97),
        "Moon" => Color32::from_rgb(218, 233, 255),
        "Mars" => Color32::from_rgb(255, 139, 115),
        "Jupiter" => Color32::from_rgb(224, 182, 145),
        "Saturn" => Color32::from_rgb(217, 210, 148),
        "Venus" => Color32::from_rgb(239, 190, 245),
        "Mercury" => Color32::from_rgb(158, 176, 190),
        _ => Color32::from_rgb(117, 221, 224),
    }
}
pub fn horizon_position(
    center: Pos2,
    radius: f32,
    azimuth: f64,
    altitude: f64,
    below: bool,
) -> Pos2 {
    let fraction = if below {
        (90. + altitude) / 90.
    } else {
        (90. - altitude) / 90.
    };
    let az = azimuth.to_radians();
    let x = az.sin() * if below { 1. } else { -1. };
    center + egui::vec2((x * fraction) as f32, (-az.cos() * fraction) as f32) * radius
}
struct MarkerStyle<'a> {
    refraction: bool,
    below: bool,
    ecliptic: bool,
    query: &'a str,
    features: &'a [Feature],
    settings: &'a Settings,
}
fn markers(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    bodies: &[BodyPosition],
    style: MarkerStyle<'_>,
    response: &egui::Response,
) {
    let MarkerStyle {
        refraction,
        below,
        ecliptic,
        query,
        features,
        settings,
    } = style;
    let mut labels: Vec<Rect> = vec![];
    for b in bodies {
        let alt = b.altitude_for(refraction);
        if !ecliptic && ((alt < 0.) != below) {
            continue;
        }
        let pos = if ecliptic {
            center
                + egui::vec2(
                    b.ecliptic_longitude.to_radians().cos() as f32,
                    -b.ecliptic_longitude.to_radians().sin() as f32,
                ) * radius
                    * 0.72
        } else {
            horizon_position(center, radius, b.azimuth, alt, below)
        };
        let hit = features
            .iter()
            .any(|f| f.domain == "Sky" && f.owner == b.name && f.matches(query, settings));
        let color = if hit {
            super::app::HIT
        } else {
            body_color(&b.name)
        };
        painter.circle_filled(
            pos,
            if b.name == "Sun" || b.name == "Moon" {
                5.
            } else {
                4.
            },
            color,
        );
        if hit {
            painter.circle_stroke(pos, 9., Stroke::new(1.5, color));
        }
        let mut label = pos + egui::vec2(9., -8.);
        if label.x + 70. > response.rect.right() {
            label.x = pos.x - 80.;
        }
        label.y = label
            .y
            .clamp(response.rect.top() + 4., response.rect.bottom() - 18.);
        for _ in 0..12 {
            let test = Rect::from_min_size(label, egui::vec2(70., 15.));
            if labels.iter().any(|r| r.intersects(test)) {
                label.y += 16.;
            } else {
                labels.push(test);
                break;
            }
        }
        if label.distance(pos) > 24. {
            painter.line_segment([pos, label], Stroke::new(0.6, color));
        }
        painter.text(
            label,
            egui::Align2::LEFT_TOP,
            &b.name,
            egui::FontId::proportional(12.),
            color,
        );
        if response.hover_pos().is_some_and(|p| p.distance(pos) < 12.) {
            response.clone().on_hover_text(format!(
                "{}\nAzimuth {:.4}° · altitude {:.4}°\nEcliptic longitude {:.4}°",
                b.name, b.azimuth, alt, b.ecliptic_longitude
            ));
        }
    }
}

pub fn draw_sky(
    ui: &mut egui::Ui,
    sky: &SkySnapshot,
    s: &Session,
    features: &[Feature],
    ecliptic: bool,
) {
    let refraction = s.settings.atmospheric_refraction;
    if ecliptic {
        ui.weak("Geocentric ecliptic of date: wheel shows longitude; latitude is in the table. This view is independent of observer location.");
        let width = ui.available_width().min(600.);
        let (response, p) = ui.allocate_painter(Vec2::splat(width), egui::Sense::hover());
        let c = response.rect.center();
        let r = width * 0.4;
        p.circle_filled(c, r, Color32::from_rgb(13, 24, 39));
        p.circle_stroke(c, r, Stroke::new(1.5, Color32::GRAY));
        for deg in (0..360).step_by(30) {
            let v = egui::vec2(
                (deg as f32).to_radians().cos(),
                -(deg as f32).to_radians().sin(),
            );
            p.line_segment([c + v * r * 0.9, c + v * r], Stroke::new(1., Color32::GRAY));
            p.text(
                c + v * (r + 15.),
                egui::Align2::CENTER_CENTER,
                format!("{deg}°"),
                egui::FontId::proportional(11.),
                Color32::LIGHT_GRAY,
            );
        }
        markers(
            &p,
            c,
            r,
            &sky.bodies,
            MarkerStyle {
                refraction,
                below: false,
                ecliptic: true,
                query: &s.query,
                features,
                settings: &s.settings,
            },
            &response,
        );
    } else {
        ui.horizontal_wrapped(|ui| {
            let width = (ui.available_width() / 2. - 8.).clamp(275., 490.);
            for below in [false, true] {
                ui.vertical(|ui| {
                    ui.label(if below {
                        "Below horizon · looking down"
                    } else {
                        "Above horizon · looking up"
                    });
                    let (response, p) =
                        ui.allocate_painter(Vec2::splat(width), egui::Sense::hover());
                    let c = response.rect.center();
                    let r = width * 0.4;
                    p.circle_filled(
                        c,
                        r,
                        if below {
                            Color32::from_rgb(22, 26, 31)
                        } else {
                            Color32::from_rgb(15, 29, 49)
                        },
                    );
                    for ring in [1., 2. / 3., 1. / 3.] {
                        p.circle_stroke(c, r * ring, Stroke::new(1., Color32::from_gray(75)));
                    }
                    for (label, x, y) in if below {
                        [("N", 0., -1.), ("E", 1., 0.), ("S", 0., 1.), ("W", -1., 0.)]
                    } else {
                        [("N", 0., -1.), ("E", -1., 0.), ("S", 0., 1.), ("W", 1., 0.)]
                    } {
                        p.line_segment(
                            [c, c + egui::vec2(x, y) * r],
                            Stroke::new(0.5, Color32::from_gray(55)),
                        );
                        p.text(
                            c + egui::vec2(x, y) * (r + 15.),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(13.),
                            Color32::LIGHT_GRAY,
                        );
                    }
                    p.text(
                        c,
                        egui::Align2::CENTER_CENTER,
                        if below {
                            "Nadir −90°"
                        } else {
                            "Zenith +90°"
                        },
                        egui::FontId::proportional(10.),
                        Color32::GRAY,
                    );
                    markers(
                        &p,
                        c,
                        r,
                        &sky.bodies,
                        MarkerStyle {
                            refraction,
                            below,
                            ecliptic: false,
                            query: &s.query,
                            features,
                            settings: &s.settings,
                        },
                        &response,
                    );
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compass_projection_distinguishes_above_and_below_horizon() {
        let c = Pos2::new(100., 100.);
        assert_eq!(horizon_position(c, 90., 0., 90., false), c);
        assert_eq!(horizon_position(c, 90., 0., -90., true), c);
        assert!(horizon_position(c, 90., 90., 0., false).x < c.x);
        assert!(horizon_position(c, 90., 90., 0., true).x > c.x);
        let front = globe_point(0., 0., 0., 0.);
        let back = globe_point(180., 0., 0., 0.);
        assert!(front.2 > 0.99 && back.2 < -0.99);
    }
    #[test]
    fn bundled_land_geometry_triangulates_without_invalid_indices() {
        let shapes = land();
        assert!(!shapes.is_empty());
        assert!(shapes.iter().map(|s| s.points.len()).sum::<usize>() > 4000);
        for s in shapes {
            assert_eq!(s.indices.len() % 3, 0);
            assert!(s.indices.iter().all(|i| *i < s.points.len()));
        }
    }
}
