use common::Game;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    game: Game,
    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    // value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self { game: Game::new() }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Say hello from the terminal to make sure logging is working (also for wasm targets).
        log::info!("App created!");

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { game } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Game").clicked() {
                        *game = Game::new();
                        ui.close_menu();
                    }

                    if ui.button("Play Game").clicked() {
                        game.play(false);
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
                ui.label(trump.to_string());
            } else if let Some(trump_suit) = &game.trump_suit {
                ui.label(trump_suit.to_string());
            } else {
                ui.label("[ Trump ]");
            }

            ui.separator();

            // trick
            ui.horizontal(|ui| {
                if let Some(trick0) = &game.trick.0 {
                    ui.label(trick0.to_string());
                } else {
                    ui.label("[ trick 0 ]");
                }

                if let Some(trick1) = &game.trick.1 {
                    ui.label(trick1.to_string());
                } else {
                    ui.label("[ trick 1 ]");
                }
            });

            ui.separator();

            // player hand
            ui.horizontal(|ui| {
                for c in game.player_hand.clone().iter().map(|c| c.to_string()) {
                    //ui.label(c.to_string());

                    if ui.button(c.to_string()).clicked() {
                        let index = &game
                            .player_hand
                            .iter()
                            .position(|p| p.to_string() == c)
                            .unwrap();
                        let card = game.player_hand.swap_remove(*index);

                        let is_forehand = game.trick.0.is_none();
                        game.play_card(card, common::EPlayer::PC, is_forehand);
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
