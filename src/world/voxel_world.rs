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
pub struct WorldNoise {
    pub terrain: SafeNode,
}

#[derive(Resource)]
pub struct VoxelStorage {
    pub chunk_len: usize,
    pub voxels: HashMap<[i32; 3], Vec<BlockType>>,
}

impl VoxelStorage {
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

    commands.insert_resource(WorldNoise { terrain: node });
    commands.insert_resource(VoxelStorage {
        chunk_len: 32,
        voxels: HashMap::<[i32; 3], Vec<BlockType>>::new(),
    });
}
