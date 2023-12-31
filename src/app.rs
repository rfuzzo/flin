use std::collections::HashMap;

use egui_extras::RetainedImage;
use egui_notify::Toasts;

use crate::{get_deck, EPlayer, Game};

static TEXTURE_SIZE: f32 = 256.0;

pub struct TemplateApp {
    game: Game,
    // this how you opt-out of serialization of a member
    //#[serde(skip)]
    toasts: Toasts,

    //#[serde(skip)]
    textures: HashMap<String, RetainedImage>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            game: Game::new(),
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
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            game,
            textures,
            toasts,
            // last_turn_time,
        } = self;

        // load all textures once
        if textures.is_empty() {
            *textures = load_textures(ctx);
        }

        // a turn in the game
        let current_time = ctx.input(|i| i.time);
        let mut allowed = false;
        if let Some(state) = &game.state {
            match state {
                crate::EGameState::None => {}
                crate::EGameState::PlayerTurn => {
                    allowed = true;
                }
                crate::EGameState::NpcTurn => {
                    let diff: f64 = current_time - game.last_turn_time;
                    if diff > 2.0 {
                        game.do_turn(toasts, current_time);
                        game.last_turn_time = ctx.input(|i| i.time);
                    }
                }
                crate::EGameState::Evaluate => {
                    let diff: f64 = current_time - game.last_turn_time;
                    if diff > 1.0 {
                        game.do_turn(toasts, current_time);
                        game.last_turn_time = ctx.input(|i| i.time);
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Game").clicked() {
                        *game = Game::new();
                        game.play(toasts, current_time);
                        ui.close_menu();
                    }

                    ui.separator();

                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // playing field
            egui::ScrollArea::vertical().show(ui, |ui| {
            // trump card
            if let Some(trump) = &game.trump_card {
                if let Some(texture) = textures.get(&trump.to_string()) {
                    let img_size = TEXTURE_SIZE * texture.size_vec2() / texture.size_vec2().y;
                    let r = ui.image(texture.texture_id(ctx), img_size);
                    r.on_hover_ui(|ui| {
                        ui.label(trump.to_string());
                    });
                } else {
                    ui.label(trump.to_string());
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
                        let r = ui.image(texture.texture_id(ctx), img_size);
                        r.on_hover_ui(|ui| {
                            ui.label(trick0.to_string());
                        });
                    } else {
                        ui.label(trick0.to_string());
                    }
                } else {
                    ui.label("[ trick 0 ]");
                }

                if let Some(trick1) = &game.trick.1 {
                    if let Some(texture) = textures.get(&trick1.to_string()) {
                        let img_size = TEXTURE_SIZE * texture.size_vec2() / texture.size_vec2().y;
                        let r = ui.image(texture.texture_id(ctx), img_size);
                        r.on_hover_ui(|ui| {
                            ui.label(trick1.to_string());
                        });
                    } else {
                        ui.label(trick1.to_string());
                    }
                } else {
                    ui.label("[ trick 1 ]");
                }
            });

            ui.separator();

            // player hand
            egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for c in game.player_hand.clone().iter().map(|c| c.to_string()) {
                    if let Some(texture) = textures.get(&c) {
                        let img_size = TEXTURE_SIZE * texture.size_vec2() / texture.size_vec2().y;

                        let w = egui::ImageButton::new(texture.texture_id(ctx), img_size);
                        let r = if !allowed {
                            ui.add_enabled(false, w)
                        } else {
                            ui.add(w)
                        };

                        if r.clicked() {
                            let index = &game
                                .player_hand
                                .iter()
                                .position(|p| p.to_string() == c)
                                .unwrap();
                            let card = game.player_hand.swap_remove(*index);

                            game.play_card(card, EPlayer::PC, current_time);
                        }

                        r.on_hover_ui(|ui| {
                            ui.label(c);
                        });
                    } else {
                        let w = egui::Button::new(c.to_string());
                        let r = if !allowed {
                            ui.add_enabled(false, w)
                        } else {
                            ui.add(w)
                        };
                        if r.clicked() {
                            let index = &game
                                .player_hand
                                .iter()
                                .position(|p| p.to_string() == c)
                                .unwrap();
                            let card = game.player_hand.swap_remove(*index);

                            game.play_card(card, EPlayer::PC, current_time);
                        }
                    }
                }
            });
            });

            // points
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Points PC: ");
                ui.label(game.get_points(EPlayer::PC).to_string());
            });

            ui.horizontal(|ui| {
                ui.label("Points NPC: ");
                ui.label(game.get_points(EPlayer::NPC).to_string());
            });

            // winner
            ui.separator();
            if let Some(winner) = game.winner {
                ui.label(format!("The winner is {}", winner));
            }
        });

            toasts.show(ctx);
        });
    }
}

//#[cfg(target_arch = "wasm32")]
fn load_textures(_ctx: &egui::Context) -> HashMap<String, RetainedImage> {
    let mut map: HashMap<String, RetainedImage> = HashMap::default();

    // include textures
    let map2: Vec<&[u8]> = vec![
        include_bytes!("../assets/hearts.unter.jpg"),
        include_bytes!("../assets/hearts.ober.jpg"),
        include_bytes!("../assets/hearts.king.jpg"),
        include_bytes!("../assets/hearts.x.jpg"),
        include_bytes!("../assets/hearts.ace.jpg"),
        include_bytes!("../assets/bells.unter.jpg"),
        include_bytes!("../assets/bells.ober.jpg"),
        include_bytes!("../assets/bells.king.jpg"),
        include_bytes!("../assets/bells.x.jpg"),
        include_bytes!("../assets/bells.ace.jpg"),
        include_bytes!("../assets/acorns.unter.jpg"),
        include_bytes!("../assets/acorns.ober.jpg"),
        include_bytes!("../assets/acorns.king.jpg"),
        include_bytes!("../assets/acorns.x.jpg"),
        include_bytes!("../assets/acorns.ace.jpg"),
        include_bytes!("../assets/leaves.unter.jpg"),
        include_bytes!("../assets/leaves.ober.jpg"),
        include_bytes!("../assets/leaves.king.jpg"),
        include_bytes!("../assets/leaves.x.jpg"),
        include_bytes!("../assets/leaves.ace.jpg"),
    ];

    for (i, c) in get_deck().iter().enumerate() {
        if let Ok(image) = RetainedImage::from_image_bytes(c.to_string(), map2[i]) {
            map.insert(c.to_string(), image);
        }
    }

    map
}
