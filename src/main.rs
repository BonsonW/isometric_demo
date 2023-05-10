use std::{env, collections::HashMap, fs::read_dir};

use bevy::prelude::*;
use rand::Rng;
use wfc_voxel::{NodeData, Solver};


static NODE_LENGTH: usize = 3;

static TILE_W: f32 = 8.;
static TILE_H: f32 = 8.;

static MAP_W: usize = 12;
static MAP_H: usize = 3;

static TILE_SCALE: f32 = 4.;

static OFFSET_Y: f32 = -200.;

#[derive(Resource)]
struct NodeDataRes(NodeData);

#[derive(Resource)]
struct SolverRes(Solver);

#[derive(Component)]
struct Tile;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            ImagePlugin::default_nearest(),
        ))
        .insert_resource(ClearColor(Color::hex("#EAD4AA").unwrap()))
        .add_startup_system(setup)
        .add_system(keyboard_input)
        .run();
}


fn setup(
    mut commands: Commands
) {
    let prelim_dir = String::from(format!("{}/assets/prelim", env::current_dir().unwrap().to_str().unwrap()));
    let node_data = NodeData::new(NODE_LENGTH, prelim_dir, HashMap::new());

    let air_nodes = node_data.asset_bits(&String::from("air")).unwrap();
    let full_nodes = node_data.asset_bits(&String::from("full")).unwrap();
    let mut initial_val = node_data.bit_mask();
    
    // remove full nodes
    for id in full_nodes.iter_ones() {
        initial_val.set(id, false);
    }
    
    let mut solver = Solver::new([MAP_W, MAP_H, MAP_W], &initial_val, &node_data, false);
    
    // collapse bottom of the map to ones that attach to full
    wfc_voxel::collapse_y_axis(
        &mut solver,
        &full_nodes,
        0,
        wfc_voxel::POS_Y,
        [0, MAP_W],
        [0, MAP_W]
    );
    
    // collapse top of the map to ones that attach to empty
    wfc_voxel::collapse_y_axis(
        &mut solver,
        &air_nodes,
        -1,
        wfc_voxel::NEG_Y,
        [0, MAP_W],
        [0, MAP_W]
    );
    
    commands.insert_resource(SolverRes(solver));
    commands.insert_resource(NodeDataRes(node_data));

    // camera
    commands.spawn(Camera2dBundle::default());
}

fn keyboard_input(
    mut commands: Commands,
    node_data: Res<NodeDataRes>,
    solver: Res<SolverRes>,
    keys: Res<Input<KeyCode>>,
    query: Query<Entity, With<Tile>>,
    asset_server: Res<AssetServer>
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    let node_data = &node_data.0;
    let mut solver =  solver.0.clone();
    let map = solver.solve();

    let mut rng = rand::thread_rng();
    let shape = solver.shape();

    for x in 0..shape[0] {
        for y in 0..shape[1] {
            for z in 0..shape[2] {
                let node_id = map[[x, y, z]];
                let asset_name = node_data.get_asset_name(&node_id);
                match asset_name.as_str() {
                    "grass" => { continue; }
                    "house_free_side" => { continue; }
                    _ => {}
                }
                
                let n_assets = read_dir(format!("{}/assets/final/{}", env::current_dir().unwrap().to_str().unwrap(), asset_name)).unwrap().count();
                
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load(format!("final/{}/castle ({}).png", asset_name, rng.gen_range(1..=n_assets))),
                        transform: get_tile_transform(x, y+1, z),
                        ..default()
                    },
                    Tile
                ));
                
                if y == 0 {
                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("final/shadow.png"),
                            transform: get_tile_transform(x+1, y, z),
                            ..default()
                        },
                        Tile
                    ));
                }
            }
        }
    }
}

fn get_tile_transform(x: usize, y: usize, z: usize) -> Transform {
    let x = x as f32;
    let y = y as f32;
    let z = z as f32;
    
    let x_coord = x - z;
    let z_coord = (x/2. + z/2.) + y;
    let y_coord = 999. - z_coord + y * 2.;
    
    let mut transform = Transform::from_xyz(
        x_coord * TILE_W * TILE_SCALE,
        z_coord * TILE_H * TILE_SCALE + OFFSET_Y,
        y_coord
    );
    
    transform.scale = Vec3::new(TILE_SCALE, TILE_SCALE, TILE_SCALE);
    transform
}