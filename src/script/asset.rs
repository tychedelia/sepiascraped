use crate::script::LispEngine;
use bevy::asset::io::Reader;
use bevy::asset::{ron, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy::prelude::*;
use bevy::utils::thiserror::Error;
use bevy::utils::HashMap;
use steel::compiler::program::RawProgramWithSymbols;

pub struct ScriptAssetPlugin;

impl Plugin for ScriptAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Script>()
            .init_asset_loader::<ScriptLoader>()
            .init_non_send_resource::<ProgramCache>()
            .add_systems(Startup, setup)
            .add_systems(Update, load_scripts);
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct ProgramCache(HashMap<AssetId<Script>, RawProgramWithSymbols>);

fn setup(mut commands: Commands, mut asset_server: ResMut<AssetServer>) {
    let script: Handle<Script> = asset_server.load("project.scm");
    commands.spawn((script,));
}

pub fn load_scripts(
    mut engine: NonSendMut<LispEngine>,
    mut program_cache: NonSendMut<ProgramCache>,
    scripts: Res<Assets<Script>>,
    mut ev_asset: EventReader<AssetEvent<Script>>,
) {
    let mut engine = engine.borrow_mut();
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::Added { id } => {
                let script = scripts.get(*id).unwrap();
                let program = match engine.emit_raw_program_no_path(script.code.clone()) {
                    Ok(program) => program,
                    Err(err) => {
                        error!("Failed to compile script: {:?}", err);
                        continue;
                    }
                };
                program_cache.insert(*id, program);
                info!("Added script: {:?}", id);
            }
            AssetEvent::Modified { id } => {
                let script = scripts.get(*id).unwrap();
                let program = match engine.emit_raw_program_no_path(script.code.clone()) {
                    Ok(program) => program,
                    Err(err) => {
                        error!("Failed to compile script: {:?}", err);
                        continue;
                    }
                };
                program_cache.insert(*id, program);
                info!("Modified script: {:?}", id);
            }
            AssetEvent::Removed { id } => {
                info!("Removed script: {:?}", id);
            }
            AssetEvent::Unused { id } => {
                info!("Unused script: {:?}", id);
            }
            AssetEvent::LoadedWithDependencies { id } => {
                info!("Loaded script with dependencies: {:?}", id);
            }
        }
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct Script {
    code: String,
}

#[derive(Default)]
struct ScriptLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ScriptLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// An [std::string::FromUtf8Error]
    #[error("Could not convert bytes to string: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}

impl AssetLoader for ScriptLoader {
    type Asset = Script;
    type Settings = ();
    type Error = ScriptLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let code = String::from_utf8(bytes)?;
            info!("Loaded script: {}", code);
            Ok(Script { code })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["scm"]
    }
}
