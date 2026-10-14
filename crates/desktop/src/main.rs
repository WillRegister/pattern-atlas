mod app;
mod geocoding;
mod history;
mod observation_editor;
mod timezones;
mod views;
use anyhow::{Context, Result};
use atlas_core::{analysis, model::Workspace};
use atlas_storage::Store;
use std::{io::Write, path::PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("Pattern Atlas could not start: {error:#}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let flag = |key: &str| -> Result<Option<String>> {
        if let Some(i) = args.iter().position(|s| s == key) {
            Ok(Some(
                args.get(i + 1).context("Flag requires a value")?.clone(),
            ))
        } else {
            Ok(None)
        }
    };
    if let Some(path) = flag("--validate-data")? {
        let file = atlas_core::bulk::ImportFile::parse(&std::fs::read_to_string(path)?)?;
        println!(
            "Valid: {} · {} entities, {} places, {} dates · batch {}",
            file.dataset.name,
            file.dataset.entities.len(),
            file.dataset.places.len(),
            file.dataset.observations.len(),
            file.batch_id
        );
        return Ok(());
    }
    // Keep the executable portable; a user may override its adjacent data directory.
    let default_dir = std::env::current_exe()?
        .parent()
        .context("Executable needs a directory")?
        .join("data");
    let data_dir = flag("--data-dir")?
        .map(PathBuf::from)
        .unwrap_or(default_dir);
    std::fs::create_dir_all(&data_dir)?;
    let mut store = Store::open(data_dir.join("atlas.sqlite"))?;
    let (mut workspace, mut revision) = match store.load()? {
        Some(saved) => saved,
        None => {
            let w = Workspace::demo();
            let r = store.save(&w, 0)?;
            (w, r)
        }
    };
    if let Some(path) = flag("--import")? {
        let mut imported: Workspace = serde_json::from_slice(&std::fs::read(path)?)?;
        imported.upgrade()?;
        revision = store.save(&imported, revision)?;
        workspace = imported;
    }
    let additions = flag("--import-data")?.or(flag("--ensure-data")?);
    if let Some(path) = additions {
        let file = atlas_core::bulk::ImportFile::parse(&std::fs::read_to_string(path)?)?;
        let exists = workspace
            .datasets
            .iter()
            .any(|d| d.import_batches.contains(&file.batch_id));
        if exists && args.iter().any(|s| s == "--ensure-data") {
            println!("Dataset batch {} is already installed", file.batch_id);
        } else {
            let dataset_id = file.apply(&mut workspace, None)?;
            let s = workspace
                .sessions
                .last_mut()
                .context("Imported session missing")?;
            if let Some(name) = flag("--session-name")? {
                s.name = name;
            }
            if let Some(name) = flag("--observer-place")? {
                let d = workspace
                    .datasets
                    .iter()
                    .find(|d| d.id == dataset_id)
                    .unwrap();
                let p = d
                    .places
                    .iter()
                    .find(|p| p.name == name)
                    .context("Observer place not found in imported dataset")?;
                s.active_place = Some(p.id.clone());
                s.locations.a = Some(p.id.clone());
                s.timezone = p.timezone.clone();
            }
            revision = store.save(&workspace, revision)?;
        }
        println!(
            "Dataset {} is ready. Existing data retained.",
            file.dataset.name
        );
    }
    if let Some(path) = flag("--export")? {
        write_new(&path, &serde_json::to_string_pretty(&workspace)?)?;
        return Ok(());
    }
    if let Some(path) = flag("--analyze")? {
        let s = &workspace.sessions[0];
        let d = workspace
            .datasets
            .iter()
            .find(|d| d.id == s.dataset_id)
            .context("Missing dataset")?;
        write_new(
            &path,
            &serde_json::to_string_pretty(&analysis::features(d, s))?,
        )?;
        return Ok(());
    }
    if args
        .iter()
        .any(|s| s == "--import" || s == "--import-data" || s == "--ensure-data")
    {
        return Ok(());
    }
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440., 920.])
            .with_min_inner_size([1000., 700.]),
        ..Default::default()
    };
    let open_session = flag("--open-session")?;
    eframe::run_native(
        "Pattern Atlas",
        native,
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(eframe::egui::Theme::Dark);
            cc.egui_ctx.set_pixels_per_point(1.15);
            let mut app = app::AtlasApp::new(workspace, revision, store, data_dir);
            if let Some(name) = open_session {
                app.open_session(&name);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
pub fn write_new(path: impl AsRef<std::path::Path>, text: &str) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    Ok(())
}
