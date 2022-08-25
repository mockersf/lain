use bevy::{asset::Asset, ecs::all_tuples, gltf::Gltf, prelude::*};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};

pub(crate) trait CloneWeak {
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
pub(crate) enum AssetState {
    Loading,
    Done,
}

pub(crate) struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(AssetState::Loading)
            .add_loading_state(
                LoadingState::new(AssetState::Loading)
                    .continue_to_state(AssetState::Done)
                    .with_collection::<RawUiAssets>()
                    .with_collection::<ZombieAssets>()
                    .with_collection::<BuildingAssets>()
                    .with_collection::<RawSceneryAssets>(),
            )
            .add_system_set(
                SystemSet::on_enter(AssetState::Done).with_system(done.exclusive_system()),
            );
    }
}

#[derive(AssetCollection)]
struct RawUiAssets {
    #[asset(path = "ui/arrowBeige_right.png")]
    selection_handle: Handle<Image>,
    #[asset(path = "fonts/kenvector_future.ttf")]
    font_main: Handle<Font>,
    #[asset(path = "fonts/mandrill.ttf")]
    font_sub: Handle<Font>,
    #[asset(path = "fonts/MaterialIcons-Regular.ttf")]
    font_material: Handle<Font>,
    #[asset(path = "ui/panel_blue.png")]
    panel_texture_handle: Handle<Image>,
    #[asset(path = "ui/buttonLong_beige.png")]
    button_texture_handle: Handle<Image>,
}

#[derive(AssetCollection)]
pub(crate) struct ZombieAssets {
    #[asset(path = "zombies/all-in-one.glb")]
    pub(crate) animations: Handle<Gltf>,
    #[asset(path = "zombies/all-in-one.glb#Scene0")]
    pub(crate) mutant: Handle<Scene>,
}

#[derive(AssetCollection)]
pub(crate) struct BuildingAssets {
    #[asset(path = "buildings/detail_crystalLarge.glb#Scene0")]
    pub(crate) crystal: Handle<Scene>,
    #[asset(path = "buildings/towerRound_sampleA.glb#Scene0")]
    pub(crate) material_tower: Handle<Scene>,
    #[asset(path = "buildings/towerSquare_sampleF.glb#Scene0")]
    pub(crate) ethereal_tower: Handle<Scene>,
    #[asset(path = "buildings/woodStructure_high.glb#Scene0")]
    pub(crate) block: Handle<Scene>,
}

#[derive(AssetCollection)]
struct RawSceneryAssets {
    #[asset(path = "scenery/tree.glb")]
    tree: Handle<Gltf>,
    #[asset(path = "scenery/trunk.glb#Scene0")]
    trunk: Handle<Scene>,
    #[asset(path = "scenery/bench.glb#Scene0")]
    bench: Handle<Scene>,
    #[asset(path = "scenery/benchDamaged.glb#Scene0")]
    bench_damaged: Handle<Scene>,
}

pub(crate) struct SceneryAssets {
    pub(crate) tree: Handle<Scene>,
    pub(crate) trunk: Handle<Scene>,
    pub(crate) bench: Handle<Scene>,
    pub(crate) bench_damaged: Handle<Scene>,
}

pub(crate) struct UiAssets {
    pub(crate) selection_handle: Handle<Image>,
    pub(crate) font_main: Handle<Font>,
    pub(crate) font_sub: Handle<Font>,
    pub(crate) font_material: Handle<Font>,
    pub(crate) panel_handle: (Handle<bevy_ninepatch::NinePatchBuilder<()>>, Handle<Image>),
    pub(crate) button_handle: Handle<crate::ui_helper::button::Button>,
}

fn done(world: &mut World) {
    info!("Done Loading Assets");
    unsafe {
        {
            let raw_ui_assets = world.remove_resource_unchecked::<RawUiAssets>().unwrap();
            let mut nine_patches = world
                .get_resource_unchecked_mut::<Assets<bevy_ninepatch::NinePatchBuilder<()>>>()
                .unwrap();
            let mut buttons = world
                .get_resource_unchecked_mut::<Assets<crate::ui_helper::button::Button>>()
                .unwrap();
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(10, 30, 10, 10);
            let panel_handle = (nine_patches.add(np), raw_ui_assets.panel_texture_handle);
            let button = crate::ui_helper::button::Button::setup(
                &mut nine_patches,
                raw_ui_assets.button_texture_handle,
            );
            let button_handle = buttons.add(button);
            world.insert_resource(UiAssets {
                selection_handle: raw_ui_assets.selection_handle,
                font_main: raw_ui_assets.font_main,
                font_sub: raw_ui_assets.font_sub,
                font_material: raw_ui_assets.font_material,
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

        {
            let scenery_assets = world
                .remove_resource_unchecked::<RawSceneryAssets>()
                .unwrap();
            let mut scenes = world.get_resource_unchecked_mut::<Assets<Scene>>().unwrap();
            let gltfs = world.get_resource::<Assets<Gltf>>().unwrap();
            let tree = gltfs.get(&scenery_assets.tree).unwrap();
            let scene = scenes.get_mut(&tree.scenes[0]).unwrap();
            let mut player = AnimationPlayer::default();
            player.play(tree.animations[0].clone()).repeat();
            scene.world.entity_mut(Entity::from_raw(1)).insert(player);
            world.insert_resource(SceneryAssets {
                tree: tree.scenes[0].clone(),
                trunk: scenery_assets.trunk,
                bench: scenery_assets.bench,
                bench_damaged: scenery_assets.bench_damaged,
            });
        }
    }
}
