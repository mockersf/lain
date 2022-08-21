use bevy::prelude::*;

#[derive(bevy::reflect::TypeUuid)]
#[uuid = "5114f317-f6a6-4436-bd2a-cb380f5eb551"]
pub struct Button {
    nine_patch: Handle<bevy_ninepatch::NinePatchBuilder<()>>,
    texture: Handle<Image>,
}

#[derive(Component)]
pub struct ButtonId<T: Into<String>>(pub T);

impl Button {
    pub fn setup(
        nine_patches: &mut Assets<bevy_ninepatch::NinePatchBuilder>,
        texture_handle: Handle<Image>,
    ) -> Button {
        let nine_patch = bevy_ninepatch::NinePatchBuilder::by_margins(7, 7, 7, 7);
        Button {
            nine_patch: nine_patches.add(nine_patch),
            texture: texture_handle,
        }
    }

    pub fn add<T>(
        &self,
        commands: &mut Commands,
        width: f32,
        height: f32,
        margin: UiRect<Val>,
        font: Handle<Font>,
        button: T,
        font_size: f32,
    ) -> Entity
    where
        T: Into<String> + Send + Sync + Copy + 'static,
    {
        let button_entity = commands
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(width), Val::Px(height)),
                    margin,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                color: UiColor(Color::NONE),
                ..Default::default()
            })
            .insert(ButtonId(button))
            .id();

        let button_content = commands
            .spawn_bundle(TextBundle {
                style: Style {
                    size: Size {
                        height: Val::Px(font_size),
                        ..Default::default()
                    },
                    margin: UiRect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                text: Text::from_section(
                    button.into(),
                    TextStyle {
                        font,
                        font_size,
                        color: crate::ui::ColorScheme::TEXT_DARK,
                        ..Default::default()
                    },
                ),
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..Default::default()
            })
            .insert(bevy::ui::FocusPolicy::Pass)
            .id();

        let patch_entity = commands
            .spawn_bundle(bevy_ninepatch::NinePatchBundle::<()> {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    size: Size::new(Val::Px(width), Val::Px(height)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    self.texture.clone(),
                    self.nine_patch.clone(),
                    button_content,
                ),
                ..Default::default()
            })
            .id();

        let interaction_overlay = commands
            .spawn_bundle(ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    margin: UiRect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    size: Size::new(Val::Px(width), Val::Px(height)),
                    ..Default::default()
                },
                color: UiColor(Color::NONE),
                ..Default::default()
            })
            .insert(bevy::ui::FocusPolicy::Pass)
            .id();

        commands
            .entity(button_entity)
            .push_children(&[patch_entity, interaction_overlay]);

        button_entity
    }
}

fn button_effect(
    interaction_query: Query<
        (&bevy::ui::widget::Button, &Interaction, &Children),
        Changed<Interaction>,
    >,
    mut image_query: Query<&mut UiColor>,
) {
    for (_button, interaction, children) in interaction_query.iter() {
        let mut material = image_query
            .get_component_mut::<UiColor>(children[children.len() - 1])
            .unwrap();
        match *interaction {
            Interaction::Clicked => {
                material.0 = Color::rgba(0., 0.2, 0.2, 0.6);
            }
            Interaction::Hovered => {
                material.0 = Color::rgba(0., 0.2, 0.2, 0.3);
            }
            Interaction::None => {
                material.0 = Color::NONE;
            }
        }
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Button>().add_system(button_effect);
    }
}
