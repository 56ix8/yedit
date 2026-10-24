//! Project links: the Help menu, the About dialog and the start screen open these. YEDIT has no
//! website, community or release feed of its own yet, so the only link is the attribution to the
//! upstream project it is based on.

use serde_json::{Value, json};

use crate::YeditApp;

/// The project YEDIT is based on (plain-text attribution; no ArtCraft marks).
pub const UPSTREAM: &str = "https://github.com/storytold/photocraft";

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`. None for now.
pub const COMMANDS: &[(&str, &str)] = &[];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut YeditApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// "Based on PhotoCraft" as a link, centred. Clicks route through [`open`] (the platform browser
/// service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut YeditApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let links = [(tl!("Based on PhotoCraft"), UPSTREAM)];
    let font = egui::FontId::proportional(12.5);
    let sep = "  ·  ";
    let gaps = links.len().saturating_sub(1) as f32;
    let width: f32 = links.iter().map(|(l, _)| ui.painter().layout_no_wrap((*l).into(), font.clone(), t.text).size().x).sum::<f32>()
        + gaps * ui.painter().layout_no_wrap(sep.into(), font.clone(), t.text).size().x;
    let mut clicked: Option<&str> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        for (i, (label, url)) in links.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new(sep).font(font.clone()).color(t.text_faint));
            }
            if ui.link(egui::RichText::new(*label).font(font.clone()).color(t.accent)).on_hover_text(*url).clicked() {
                clicked = Some(url);
            }
        }
    });
    if let Some(url) = clicked {
        open(app, ui.ctx(), url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links (Help menu, start-page links) must open in the browser. They route
        // through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = YeditApp::new(yedit_engine::Session::new(), services);
        let ctx = egui::Context::default();
        open(&mut app, &ctx, UPSTREAM);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(UPSTREAM));
        assert_eq!(app.ui.status, format!("Opened {UPSTREAM}"));
    }

    #[test]
    fn help_menu_has_no_external_links_or_update_check() {
        let app = YeditApp::new(yedit_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert_eq!(help, ["help.systemInfo", "help.about"]);
        assert!(COMMANDS.is_empty());
    }
}
