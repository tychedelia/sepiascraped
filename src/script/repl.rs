use crate::Sets;
use bevy::ecs::query::QuerySingleError;
pub use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext};

pub struct ScriptReplPlugin;

impl Plugin for ScriptReplPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (repl_ui, create_repl).in_set(Sets::Ui));
    }
}

#[derive(Component, Clone, Debug)]
pub struct ScriptRepl;

fn repl_ui(mut egui_ctx: Query<&mut EguiContext, Without<PrimaryWindow>>) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else {
        return;
    };

    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        ui.horizontal(|ui| {
            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
            let indentation = 8.0 * ui.fonts(|f| f.glyph_width(&font_id, ' '));
            let item_spacing = ui.spacing_mut().item_spacing;
            ui.add_space(indentation - item_spacing.x);

            egui::Grid::new("code_samples")
                .striped(true)
                .num_columns(2)
                .min_col_width(16.0)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                });
        });

        ui.separator();

        ui.separator();

        let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        ui.collapsing("Theme", |ui| {
            theme.ui(ui);
            theme.store_in_memory(ui.ctx());
        });
    });
}

fn create_repl(mut commands: Commands, repl_q: Query<Entity, (With<ScriptRepl>, Without<Window>)>) {
    match repl_q.get_single() {
        Ok(entity) => {
            commands.entity(entity).insert(Window {
                title: "repl".to_owned(),
                ..default()
            });
        }
        Err(err) => match err {
            QuerySingleError::NoEntities(_) => {}
            QuerySingleError::MultipleEntities(_) => {
                panic!("Multiple repl entities found")
            }
        },
    }
}
