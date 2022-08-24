use bevy::{asset::Asset, ecs::all_tuples, gltf::Gltf, prelude::*};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};

pub trait CloneWeak {
    fn clone_weak(&self) -> Self;
}

impl<H: CloneWeak> CloneWeak for Option<H> {
    fn clone_weak(&self) -> Self {
        self.as_ref().map(|h| h.clone_weak())
    }
}

impl<T: Asset> CloneWeak for Handle<T> {
    fn clone_weak(&self) -> Self {
        self.clone_weak()
    }
}

macro_rules! impl_tuple_handle_clone_weak {
    ($($name: ident),*) => {
        impl<$($name: CloneWeak,)*>  CloneWeak for ($($name,)*) {
            #[allow(clippy::unused_unit)]
            fn clone_weak(&self) -> Self {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name.clone_weak(),)*)
            }
        }
    }
}

all_tuples!(impl_tuple_handle_clone_weak, 0, 15, H);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AssetState {
    Loading,
    Done,
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(AssetState::Loading)
            .add_loading_state(
                LoadingState::new(AssetState::Loading)
                    .continue_to_state(AssetState::Done)
                    .with_collection::<RawMenuAssets>()
                    .with_collection::<ZombieAssets>()
                    .with_collection::<BuildingAssets>(),
            )
            .add_system_set(
                SystemSet::on_enter(AssetState::Done).with_system(done.exclusive_system()),
            );
    }
}

#[derive(AssetCollection)]
struct RawMenuAssets {
    #[asset(path = "ui/arrowBeige_right.png")]
    selection_handle: Handle<Image>,
    #[asset(path = "fonts/kenvector_future.ttf")]
    font_main_handle: Handle<Font>,
    #[asset(path = "fonts/mandrill.ttf")]
    font_sub_handle: Handle<Font>,
    #[asset(path = "ui/panel_blue.png")]
    panel_texture_handle: Handle<Image>,
    #[asset(path = "ui/buttonLong_beige.png")]
    button_texture_handle: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct ZombieAssets {
    #[asset(path = "zombies/all-in-one.glb")]
    pub animations: Handle<Gltf>,
    #[asset(path = "zombies/all-in-one.glb#Scene0")]
    pub mutant: Handle<Scene>,
}

#[derive(AssetCollection)]
pub struct BuildingAssets {
    #[asset(path = "buildings/detail_crystalLarge.glb#Scene0")]
    pub crystal: Handle<Scene>,
}

pub struct MenuAssets {
    pub selection_handle: Handle<Image>,
    pub font_main_handle: Handle<Font>,
    pub font_sub_handle: Handle<Font>,
    pub panel_handle: (Handle<bevy_ninepatch::NinePatchBuilder<()>>, Handle<Image>),
    pub button_handle: Handle<crate::ui::button::Button>,
}

fn done(world: &mut World) {
    info!("Done Loading Assets");
    unsafe {
        {
            let raw_menu_assets = world.remove_resource_unchecked::<RawMenuAssets>().unwrap();
            let mut nine_patches = world
                .get_resource_unchecked_mut::<Assets<bevy_ninepatch::NinePatchBuilder<()>>>()
                .unwrap();
            let mut buttons = world
                .get_resource_unchecked_mut::<Assets<crate::ui::button::Button>>()
                .unwrap();
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(10, 30, 10, 10);
            let panel_handle = (nine_patches.add(np), raw_menu_assets.panel_texture_handle);
            let button = crate::ui::button::Button::setup(
                &mut nine_patches,
                raw_menu_assets.button_texture_handle,
            );
            let button_handle = buttons.add(button);
            world.insert_resource(MenuAssets {
                selection_handle: raw_menu_assets.selection_handle,
                font_main_handle: raw_menu_assets.font_main_handle,
                font_sub_handle: raw_menu_assets.font_sub_handle,
                panel_handle,
                button_handle,
            });
        }

        {
            let zombie_assets = world.get_resource_unchecked_mut::<ZombieAssets>().unwrap();
            let mut scenes = world.get_resource_unchecked_mut::<Assets<Scene>>().unwrap();
            let gltfs = world.get_resource::<Assets<Gltf>>().unwrap();
            let scene = scenes.get_mut(&zombie_assets.mutant).unwrap();
            let animations = gltfs.get(&zombie_assets.animations).unwrap();
            let mut player = AnimationPlayer::default();
            player
                .play(animations.named_animations["Walk3"].clone_weak())
                .repeat();
            scene.world.entity_mut(Entity::from_raw(1)).insert(player);
        }
    }
}
