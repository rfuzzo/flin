use std::{collections::HashMap, path::Path};

use common::{get_deck, Game};
use egui_notify::Toasts;

static TEXTURE_SIZE: f32 = 256.0;

pub struct TemplateApp {
    game: Game,
    // this how you opt-out of serialization of a member
    //#[serde(skip)]
    toasts: Toasts,

    //#[serde(skip)]
    textures: HashMap<String, egui::TextureHandle>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            game: Game::new(false, None, None),
            textures: HashMap::default(),
            toasts: Toasts::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Say hello from the terminal to make sure logging is working (also for wasm targets).
        log::info!("App created!");

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    pub fn notify(msg: &str, level: log::Level) {
        match level {
            log::Level::Error => {
                //self.toasts.error(msg);
                log::error!("{}", msg);
            }
            log::Level::Warn => log::warn!("{}", msg),
            log::Level::Info => log::info!("{}", msg),
            log::Level::Debug => log::debug!("{}", msg),
            log::Level::Trace => log::trace!("{}", msg),
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    #[cfg(target_arch = "wasm32")]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            game,
            textures,
            toasts,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    #[cfg(not(target_arch = "wasm32"))]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            game,
            textures,
            toasts,
        } = self;

        // load all textures once
        if textures.is_empty() {
            *textures = load_textures(ctx);
        }

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Game").clicked() {
                        *game = Game::new(false, Some(Self::notify), None);
                        ui.close_menu();
                    }

                    if ui.button("Play Game").clicked() {
                        game.play();
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // playing field

            // trump card
            if let Some(trump) = &game.trump_card {
                if let Some(texture) = textures.get(&trump.to_string()) {
                    let img_size = TEXTURE_SIZE * texture.size_vec2() / texture.size_vec2().y;
                    let r = ui.image(texture, img_size);
                    r.on_hover_ui(|ui| {
                        ui.label(trump.to_string());
                    });
                }
            } else if let Some(trump_suit) = &game.trump_suit {
                ui.label(trump_suit.to_string());
            } else {
                ui.label("[ Trump ]");
            }

            ui.separator();

            // trick
            ui.horizontal(|ui| {
                if let Some(trick0) = &game.trick.0 {
                    if let Some(texture) = textures.get(&trick0.to_string()) {
                        let img_size = TEXTURE_SIZE * texture.size_vec2() / texture.size_vec2().y;
                        let r = ui.image(texture, img_size);
                        r.on_hover_ui(|ui| {
                            ui.label(trick0.to_string());
                        });
                    }
                } else {
                    ui.label("[ trick 0 ]");
                }

                if let Some(trick1) = &game.trick.1 {
                    if let Some(texture) = textures.get(&trick1.to_string()) {
                        let img_size = TEXTURE_SIZE * texture.size_vec2() / texture.size_vec2().y;
                        let r = ui.image(texture, img_size);
                        r.on_hover_ui(|ui| {
                            ui.label(trick1.to_string());
                        });
                    }
                } else {
                    ui.label("[ trick 1 ]");
                }
            });

            ui.separator();

            // player hand
            ui.horizontal(|ui| {
                for c in game.player_hand.clone().iter().map(|c| c.to_string()) {
                    if let Some(texture) = textures.get(&c) {
                        let img_size = TEXTURE_SIZE * texture.size_vec2() / texture.size_vec2().y;

                        let w = egui::ImageButton::new(texture, img_size);
                        let r = ui.add(w);
                        if r.clicked() {
                            //if ui.button(c.to_string()).clicked() {
                            let index = &game
                                .player_hand
                                .iter()
                                .position(|p| p.to_string() == c)
                                .unwrap();
                            let card = game.player_hand.swap_remove(*index);

                            let is_forehand = game.trick.0.is_none();
                            game.play_card(card, common::EPlayer::PC, is_forehand);
                        }
                        r.on_hover_ui(|ui| {
                            ui.label(c);
                        });
                    }
                }
            });

            // points
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Points PC: ");
                ui.label(game.get_points(common::EPlayer::PC).to_string());
            });

            ui.horizontal(|ui| {
                ui.label("Points NPC: ");
                ui.label(game.get_points(common::EPlayer::NPC).to_string());
            });

            // winner
            ui.separator();
            if let Some(winner) = game.winner {
                ui.label(format!("The winner is {}", winner));
            }
        });
    }
}

fn load_image_from_path<P>(path: P) -> Result<egui::ColorImage, image::ImageError>
where
    P: AsRef<Path>,
{
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn load_textures(ctx: &egui::Context) -> HashMap<String, egui::TextureHandle> {
    let mut map: HashMap<String, egui::TextureHandle> = HashMap::default();
    let path = std::env::current_dir().unwrap();

    for c in get_deck() {
        let path = path
            .join("assets")
            .join(format!("{}.jpg", c.to_string().to_lowercase()));
        if path.exists() {
            if let Ok(image) = load_image_from_path(path) {
                let texture = ctx.load_texture(c.to_string(), image, Default::default());
                map.insert(c.to_string(), texture);
            }
        }
    }
    map
}
