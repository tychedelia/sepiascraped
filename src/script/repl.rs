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

#[derive(Component)]
pub struct ScriptReplCode {
    pub history: String,
    pub input: String,
}

fn repl_ui(mut egui_ctx: Query<(&mut EguiContext, &mut ScriptReplCode), Without<PrimaryWindow>>) {
    let Ok((mut ctx, mut editor)) = egui_ctx.get_single_mut() else {
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
                .show(ui, |ui| {});
        });

        ui.separator();

        ui.separator();

        let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "javascript");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui_extras::syntax_highlighting::code_view_ui(ui, &theme, &editor.history.clone(), "js");

        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut editor.input)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .lock_focus(true)
                    .layouter(&mut layouter),
            );
            if ui.button(">").clicked() {
                let new_line = editor.input.clone();
                editor.history.push_str(&new_line);
                editor.history.push('\n');
                editor.input.clear();
            }
        });
    });
}

fn create_repl(mut commands: Commands, repl_q: Query<Entity, (With<ScriptRepl>, Without<Window>)>) {
    match repl_q.get_single() {
        Ok(entity) => {
            commands.entity(entity).insert((
                Window {
                    title: "repl".to_owned(),
                    ..default()
                },
                ScriptReplCode {
                    history: "".to_string(),
                    input: "".to_string(),
                },
            ));
        }
        Err(err) => match err {
            QuerySingleError::NoEntities(_) => {}
            QuerySingleError::MultipleEntities(_) => {
                panic!("Multiple repl entities found")
            }
        },
    }
}
