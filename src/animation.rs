use bevy::{prelude::*, utils::HashMap};

pub struct SpookyAnimationPlugin;

impl Plugin for SpookyAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationLibrary::default())
            .add_event::<AnimationTransitionEvent>()
            .add_system(assign_animation_controllers)
            .add_system(read_animation_events)
            .add_system(transfer_animations.after(read_animation_events));
    }
}

pub struct AnimationTransitionEvent {
    pub entity_id: Entity,
    pub animation_name: String,
}

#[derive(Component)]
pub struct AnimationMarker {
    pub collection: String,
    pub starting_clip: String,
}

impl AnimationMarker {
    pub fn new(collection: &str, starting_clip: &str) -> Self {
        AnimationMarker {
            collection: collection.to_string(),
            starting_clip: starting_clip.to_string(),
        }
    }
}

#[derive(Component)]
pub struct AnimationController {
    pub parent_entity_id: Entity,
    pub animation_collection_name: String,
    pub current_clip: String,
}

#[derive(Resource, Default)]
pub struct AnimationLibrary(HashMap<String, HashMap<String, Handle<AnimationClip>>>);

impl AnimationLibrary {
    pub fn insert(
        &mut self,
        collection_name: &str,
        animation_name: &str,
        animation_clip: Handle<AnimationClip>,
    ) {
        if !self.0.contains_key(collection_name) {
            let mut new_collection: HashMap<String, Handle<AnimationClip>> = HashMap::new();
            new_collection.insert(animation_name.to_owned(), animation_clip);
            self.0.insert(collection_name.to_owned(), new_collection);
        } else {
            let target_collection = self.0.get_mut(collection_name).unwrap();
            target_collection.insert(animation_name.to_owned(), animation_clip);
        }
    }

    pub fn get(
        &self,
        collection_name: &str,
        animation_name: &str,
    ) -> Option<Handle<AnimationClip>> {
        if !self.0.contains_key(collection_name) {
            None
        } else {
            let target_collection = self.0.get(collection_name).unwrap();
            if !target_collection.contains_key(animation_name) {
                None
            } else {
                Some(target_collection.get(animation_name).unwrap().clone_weak())
            }
        }
    }
}

pub fn read_animation_events(
    mut animation_transition_reader: EventReader<AnimationTransitionEvent>,
    mut animation_controller_query: Query<&mut AnimationController>,
) {
    for event in animation_transition_reader.iter() {
        for mut controller in &mut animation_controller_query {
            if controller.parent_entity_id == event.entity_id {
                if controller.current_clip != event.animation_name {
                    controller.current_clip = event.animation_name.clone();
                }
            }
        }
    }
}

pub fn transfer_animations(
    animation_library: Res<AnimationLibrary>,
    mut query: Query<
        (&AnimationController, &mut AnimationPlayer),
        Or<(Changed<AnimationController>, Added<AnimationController>)>,
    >,
) {
    for (controller, mut player) in &mut query {
        player
            .play(
                animation_library
                    .get(
                        &controller.animation_collection_name,
                        &controller.current_clip,
                    )
                    .unwrap(),
            )
            .repeat();
    }
}

fn assign_animation_controllers(
    mut commands: Commands,
    marker_query: Query<(Entity, &AnimationMarker)>,
    child_query: Query<&Children>,
    animation_player_query: Query<Entity, (With<AnimationPlayer>, Without<AnimationController>)>,
) {
    if !animation_player_query.is_empty() {
        for (entity, marker) in &marker_query {
            for anim_entity in &animation_player_query {
                for descendant in child_query.iter_descendants(entity) {
                    if descendant == anim_entity {
                        commands.entity(anim_entity).insert(AnimationController {
                            parent_entity_id: entity,
                            animation_collection_name: marker.collection.clone(),
                            current_clip: marker.starting_clip.clone(),
                        });
                    }
                }
            }
        }
    }
}
