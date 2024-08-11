use std::marker::PhantomData;

use bevy_math::{Rect, Vec2, Vec3A};
use bevy_render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy_render::render_asset::RenderAssetUsages;
use bevy_render::view::VisibilitySystems;
use bevy_utils::HashMap;

use bevy_color::prelude::*;
use bevy_ecs::prelude::*;
use bevy_app::prelude::*;
use bevy_render::prelude::*;
use bevy_pbr::prelude::*;
use bevy_sprite::Anchor;
use bevy_transform::prelude::*;
use bevy_asset::prelude::*;
use bevy_reflect::prelude::*;

/// Adds the ability to render sprites in a 3D space.
pub struct Sprite3dPlugin<M: SizedMaterial = StandardMaterial> {
    phantom: PhantomData<M>,
}

impl<M: SizedMaterial> Default for Sprite3dPlugin<M> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<M: SizedMaterial> Plugin for Sprite3dPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshBatch<M>>();
        app.configure_sets(PostUpdate, Sprite3dSystems
            .after(TransformSystem::TransformPropagate)
            .after(VisibilitySystems::VisibilityPropagate),
        );
        app.add_systems(
            PostUpdate,
            batch_sprites::<M>.in_set(Sprite3dSystems)
        );
    }
}

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Sprite3dSystems;

fn batch_sprites<M: SizedMaterial>(
    mut commands: Commands,
    mut sprites: Query<(&Sprite3d, &GlobalTransform, &Handle<M>, &InheritedVisibility)>,
    mut mesh_batch: ResMut<MeshBatch<M>>,
    materials: Res<Assets<M>>,
    mut meshes: ResMut<Assets<Mesh>>,
    images: Res<Assets<Image>>,
) {
    let mesh_batch = &mut *mesh_batch;

    // Clears mesh batch
    mesh_batch.remove_unloaded_meshes(&materials, &mut commands);
    mesh_batch.clear_meshes(&mut meshes);

    // Submits sprite data to mesh batch
    for (sprite, sprite_transf, sprite_mat, sprite_vis) in &mut sprites {
        let visible = sprite_vis.get();
        if !visible { continue };
        mesh_batch.submit(
            sprite,
            sprite_transf,
            sprite_mat,
            &mut meshes,
            &materials, 
            &images,
            &mut commands
        );
    }
}

#[derive(Bundle, Clone, PartialEq, Default, Debug)]
pub struct Sprite3dBundle<M: SizedMaterial> {
    pub sprite3d: Sprite3d,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub material: Handle<M>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Component, Reflect, Clone, PartialEq, Default, Debug)]
pub struct Sprite3d {
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub custom_size: Option<Vec2>,
    pub rect: Option<Rect>,
    pub anchor: Anchor,
}

#[derive(Resource, Reflect, Debug)]
struct MeshBatch<M: SizedMaterial> {
    meshes: HashMap<Handle<M>, (Entity, Handle<Mesh>)>,
}

impl<M: SizedMaterial> Default for MeshBatch<M> {
    fn default() -> Self {
        Self {
            meshes: Default::default(),
        }
    }
}

impl<M: SizedMaterial> MeshBatch<M> {

    // Adds sprite vertex data to a mesh that is compatible with the sprite's material.
    fn submit(
        &mut self,
        sprite: &Sprite3d,
        sprite_transf: &GlobalTransform,
        sprite_mat_handle: &Handle<M>,
        meshes: &mut Assets<Mesh>,
        materials: &Assets<M>,
        images: &Assets<Image>,
        commands: &mut Commands,
    ) {

        // Gets sizes of sprite
        let Some(sprite_mat) = materials.get(sprite_mat_handle) else { return };
        let Some(sprite_mat_size) = sprite_mat.size(images) else { return };
        let sprite_size = match (sprite.custom_size, sprite.rect) {
            (Some(custom_size), _)  => custom_size,
            (None, Some(rect))       => rect.size(),
            _ => sprite_mat_size,
        };

        // Gets mesh associated with sprite's material, or creates it on the fly.
        let (_mesh_entity, mesh_handle) = self.meshes
            .entry(sprite_mat_handle.clone_weak())
            .or_insert_with(|| {
                let mesh_handle = meshes.add(create_mesh());
                let mesh_entity = commands.spawn(MaterialMeshBundle {
                    mesh: mesh_handle.clone(),
                    material: sprite_mat_handle.clone_weak(),
                    ..Default::default()
                }).id();
                (mesh_entity, mesh_handle)

            });
        let mesh = meshes.get_mut(mesh_handle).unwrap();

        // Submits sprite vertex data to mesh
        submit_sprite(mesh, sprite, sprite_transf, sprite_mat_size, sprite_size);
    }

    fn remove_unloaded_meshes(&mut self, materials: &Assets<M>, commands: &mut Commands,) {
        self.meshes.retain(|mat_handle, (mesh_entity, _)| {
            if materials.contains(mat_handle) { true }
            else {
                commands.entity(*mesh_entity).despawn();
                false
            }
        });
    }

    fn clear_meshes(&mut self, mesh_assets: &mut Assets<Mesh>) {
        for (_mesh_entity, mesh_handle) in self.meshes.values_mut() {
            let mesh = mesh_assets.get_mut(mesh_handle).unwrap();
            clear_mesh(mesh);
        }
    }
}

fn create_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_indices(Indices::U32(vec![]));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(vec![]));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(vec![]));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(vec![]));
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, VertexAttributeValues::Float32x4(vec![]));
    mesh
}

fn submit_sprite(
    mesh: &mut Mesh,
    sprite: &Sprite3d,
    sprite_transf: &GlobalTransform,
    sprite_mat_size: Vec2,
    sprite_size: Vec2,
) {
    let isize = 1.0 / sprite_mat_size;
    let hsize = sprite_size * 0.5;
    let transf = sprite_transf.affine();
    let offset = -sprite.anchor.as_vec() * sprite_size;
    let offset = Vec3A::new(offset.x, offset.y, 0.0);
    
    let bl = transf.transform_point3a(Vec3A::new(-hsize.x, -hsize.y, 0.0) + offset);
    let br = transf.transform_point3a(Vec3A::new(hsize.x, -hsize.y, 0.0) + offset);
    let tr = transf.transform_point3a(Vec3A::new(hsize.x, hsize.y, 0.0) + offset);
    let tl = transf.transform_point3a(Vec3A::new(-hsize.x, hsize.y, 0.0) + offset);
    let norm = (br - bl).cross(tl - bl).normalize();
    
    let (mut bl_uv, mut br_uv, mut tr_uv, mut tl_uv) = match sprite.rect {
        Some(rect) => {
            (
                [rect.min.x*isize.x, rect.max.y*isize.y],
                [rect.max.x*isize.x, rect.max.y*isize.y],
                [rect.max.x*isize.x, rect.min.y*isize.y],
                [rect.min.x*isize.x, rect.min.y*isize.y],
            )
        },
        None => ([0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]),
    };
    if sprite.flip_x {
        std::mem::swap(&mut tl_uv[0], &mut tr_uv[0]);
        std::mem::swap(&mut bl_uv[0], &mut br_uv[0]);
    }
    if sprite.flip_y {
        std::mem::swap(&mut tl_uv[1], &mut bl_uv[1]);
        std::mem::swap(&mut tr_uv[1], &mut br_uv[1]);
    }

    let mesh_positions = match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => panic!("Missing mesh positions"),
    };
    let i = mesh_positions.len() as u32;
    mesh_positions.extend([bl.to_array(), br.to_array(), tr.to_array(), tl.to_array()]);

    let mesh_uvs = match mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => values,
        _ => panic!("Missing mesh uvs"),
    };
    mesh_uvs.extend([bl_uv, br_uv, tr_uv, tl_uv]);

    let mesh_norms = match mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => panic!("Missing mesh normals"),
    };
    mesh_norms.extend([norm.to_array(), norm.to_array(), norm.to_array(), norm.to_array()]);

    let mesh_colors = match mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => values,
        _ => panic!("Missing mesh colors"),
    };
    let color = sprite.color.to_linear().to_f32_array();
    mesh_colors.extend([color, color, color, color]);

    let mesh_indices = match mesh.indices_mut() {
        Some(Indices::U32(mesh_indices)) => mesh_indices,
        _ => panic!("Missing mesh indices"),
    };
    mesh_indices.extend([i+0, i+1, i+2, i+2, i+3, i+0]);
}

fn clear_mesh(mesh: &mut Mesh) {
    match mesh.indices_mut() {
        Some(Indices::U16(indices)) => indices.clear(),
        Some(Indices::U32(indices)) => indices.clear(),
        None => {},
    }
    for (_, values) in mesh.attributes_mut() {
        match values {
            VertexAttributeValues::Float32(values)      => values.clear(),
            VertexAttributeValues::Sint32(values)       => values.clear(),
            VertexAttributeValues::Uint32(values)       => values.clear(),
            VertexAttributeValues::Float32x2(values)    => values.clear(),
            VertexAttributeValues::Sint32x2(values)     => values.clear(),
            VertexAttributeValues::Uint32x2(values)     => values.clear(),
            VertexAttributeValues::Float32x3(values)    => values.clear(),
            VertexAttributeValues::Sint32x3(values)     => values.clear(),
            VertexAttributeValues::Uint32x3(values)     => values.clear(),
            VertexAttributeValues::Float32x4(values)    => values.clear(),
            VertexAttributeValues::Sint32x4(values)     => values.clear(),
            VertexAttributeValues::Uint32x4(values)     => values.clear(),
            VertexAttributeValues::Sint16x2(values)     => values.clear(),
            VertexAttributeValues::Snorm16x2(values)    => values.clear(),
            VertexAttributeValues::Uint16x2(values)     => values.clear(),
            VertexAttributeValues::Unorm16x2(values)    => values.clear(),
            VertexAttributeValues::Sint16x4(values)     => values.clear(),
            VertexAttributeValues::Snorm16x4(values)    => values.clear(),
            VertexAttributeValues::Uint16x4(values)     => values.clear(),
            VertexAttributeValues::Unorm16x4(values)    => values.clear(),
            VertexAttributeValues::Sint8x2(values)      => values.clear(),
            VertexAttributeValues::Snorm8x2(values)     => values.clear(),
            VertexAttributeValues::Uint8x2(values)      => values.clear(),
            VertexAttributeValues::Unorm8x2(values)     => values.clear(),
            VertexAttributeValues::Sint8x4(values)      => values.clear(),
            VertexAttributeValues::Snorm8x4(values)     => values.clear(),
            VertexAttributeValues::Uint8x4(values)      => values.clear(),
            VertexAttributeValues::Unorm8x4(values)     => values.clear(),
        }
    }
}

/// Material that is able to report its size in pixels.
pub trait SizedMaterial: Material {
    fn size(&self, images: &Assets<Image>) -> Option<Vec2>;
}

impl SizedMaterial for StandardMaterial {
    /// Attempts to report its size as the size of its base color texture.
    fn size(&self, images: &Assets<Image>) -> Option<Vec2> {
        match &self.base_color_texture {
            Some(base_color_texture) => match images.get(base_color_texture) {
                Some(base_color_texture) => Some(base_color_texture.size_f32()),
                None => None,
            },
            None => None,
        }
    }
}