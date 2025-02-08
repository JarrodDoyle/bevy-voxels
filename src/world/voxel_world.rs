use bevy::{prelude::*, utils::HashMap};
use fastnoise2::SafeNode;

use crate::{assets::BlockType, screens::Screen, AppSet};

pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut App) {
        // !HACK: Putting this in TickTimers is yuck
        app.add_systems(OnEnter(Screen::Gameplay), setup.in_set(AppSet::TickTimers));
    }
}

#[derive(Resource)]
pub struct VoxelWorld {
    pub world_name: String,
    pub terrain_noise: SafeNode,
    pub terrain_frequency: f32,
    pub terrain_seed: i32,
    pub chunk_len: usize,
    pub voxels: HashMap<[i32; 3], Vec<BlockType>>,
}

impl VoxelWorld {
    pub fn get_voxel(
        &self,
        chunk_pos: &[i32; 3],
        local_x: usize,
        local_y: usize,
        local_z: usize,
    ) -> Option<BlockType> {
        self.voxels
            .get(chunk_pos)
            .map(|chunk| chunk[self.local_pos_to_idx(local_x, local_y, local_z)])
    }

    pub fn set_voxel(
        &mut self,
        chunk_pos: &[i32; 3],
        local_x: usize,
        local_y: usize,
        local_z: usize,
        block_type: BlockType,
    ) {
        let idx = self.local_pos_to_idx(local_x, local_y, local_z);
        if let Some(chunk) = self.voxels.get_mut(chunk_pos) {
            chunk[idx] = block_type;
        }
    }

    pub fn get_chunk(&self, chunk_pos: &[i32; 3]) -> Option<&Vec<BlockType>> {
        self.voxels.get(chunk_pos)
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: &[i32; 3]) -> Option<&mut Vec<BlockType>> {
        self.voxels.get_mut(chunk_pos)
    }

    pub fn load_chunk(&mut self, chunk_pos: &[i32; 3], chunk_voxels: Vec<BlockType>) {
        assert_eq!(
            chunk_voxels.len(),
            self.chunk_len * self.chunk_len * self.chunk_len
        );

        self.voxels.insert(*chunk_pos, chunk_voxels);
    }

    pub fn local_pos_to_idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.chunk_len + z * self.chunk_len * self.chunk_len
    }
}

fn setup(mut commands: Commands) {
    let encoded_node_tree = "DQADAAAAAAAAQCkAAAAAAD8AAAAAAA==";
    let node = SafeNode::from_encoded_node_tree(encoded_node_tree).unwrap();

    commands.insert_resource(VoxelWorld {
        world_name: "Test World".to_string(),
        terrain_noise: node,
        terrain_frequency: 0.005,
        terrain_seed: 1338,
        chunk_len: 32,
        voxels: HashMap::<[i32; 3], Vec<BlockType>>::new(),
    });
}
