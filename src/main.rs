use bevy::{core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping}, prelude::*, sprite::Mesh2d, utils::hashbrown::HashMap};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Component)]
struct Agent {
    counter_a: u32,
    counter_b: u32,
    angle: f32,
    speed: f32,
    trigger: bool,
    destination: u8,
    ring: Entity,
    trigger_timer: f32
}

const DESTINATION_A: u8 = 0;
const DESTINATION_B: u8 = 1;

#[derive(Component)]
struct PointA;

#[derive(Component)]
struct PointB;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        tonemapping: Tonemapping::TonyMcMapface,
        ..default()
    }, BloomSettings::default()));
    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((
        ColorMesh2dBundle {
            material: colors.add(Color::YELLOW),
            mesh: meshes.add(Mesh::from(Circle::new(30.0))).into(),
            transform: Transform::from_xyz(-700.0, 0.0, 0.0),
            ..default()
        },
        PointA
    ));

    commands.spawn((
        ColorMesh2dBundle {
            material: colors.add(Color::GREEN),
            mesh: meshes.add(Mesh::from(Circle::new(30.0))).into(),
            transform: Transform::from_xyz(700.0, 0.0, 0.0),
            ..default()
        },
        PointB
    ));

    for x in -3..=3 {
        for y in -3..=3 {
            let ring = commands.spawn((
                ColorMesh2dBundle {
                    material: colors.add(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    mesh: meshes.add(Mesh::from(Circle::new(15.0))).into(),
                    ..default()
                },
            )).id();

            let parent = commands.spawn((
                ColorMesh2dBundle {
                    material: colors.add(Color::GRAY),
                    mesh: meshes.add(Mesh::from(Circle::new(2.0))).into(),
                    transform: Transform::from_xyz(
                        x as f32 * 90.0 + rand::thread_rng().gen_range(-15.0..=15.0),
                        y as f32 * 40.0 + rand::thread_rng().gen_range(-15.0..=15.0),
                        0.0,
                    ),
                    ..default()
                },
                Agent {
                    counter_a: 0,
                    counter_b: 0,
                    angle: rand::thread_rng().gen_range(0.0..std::f32::consts::PI*2.0),
                    speed: rand::thread_rng().gen_range(200.0..=297.0),
                    trigger: false,
                    destination: if rand::thread_rng().gen_bool(0.5) {DESTINATION_A} else {DESTINATION_B},
                    ring,
                    trigger_timer: 0.0
                },
            )).id();

            commands.entity(ring).set_parent(parent);
        }
    }
}

fn update(
    mut colors: ResMut<Assets<ColorMaterial>>,
    mut agents: Query<(Entity, &mut Agent, &mut Transform), (Without<PointA>, Without<PointB>)>,
    mut rings: Query<&mut Handle<ColorMaterial>>,
    point_a: Query<&Transform, (With<PointA>, Without<PointB>)>,
    point_b: Query<&Transform, (With<PointB>, Without<PointA>)>,
    time: Res<Time>,
) {
    let point_a = point_a.single();
    let point_b = point_b.single();

    let positions: Vec<_> = agents.iter().map(|x| (x.0, x.2.translation)).collect();

    for (entity, mut agent, transform) in agents.iter_mut() {
        agent.trigger_timer -= time.delta_seconds();

        if agent.trigger_timer > 0.0 {continue}

        if transform.translation.distance_squared(point_a.translation) <= 900.0 {
            agent.counter_a = 0;
            agent.trigger = true;
            agent.trigger_timer = 0.5;

            if agent.destination == DESTINATION_A {
                agent.destination = DESTINATION_B;
            }
        }

        if transform.translation.distance_squared(point_b.translation) <= 900.0 {
            agent.counter_b = 0;
            agent.trigger = true;
            agent.trigger_timer = 0.5;

            if agent.destination == DESTINATION_B {
                agent.destination = DESTINATION_A;
            }
        }
    }

    let mut triggers = vec![];

    for (entity, mut agent, transform) in agents.iter_mut() {
        if !agent.trigger {continue}

        triggers.push((transform.translation, agent.counter_a, agent.counter_b));

        agent.trigger = false;
    }

    for (pos, counter_a, counter_b) in triggers {
        for (e, t) in &positions {
            if *t == pos {continue}

            if pos.distance_squared(*t) > 350.0 * 350.0 {continue}

            let mut agent = agents.get_mut(*e).unwrap();

            if agent.1.counter_a > counter_a {
                agent.1.counter_a = counter_a + 350;

                if agent.1.trigger_timer <= 0.0 {
                    agent.1.trigger = true;
                    agent.1.trigger_timer = 0.5;
                }

                if agent.1.destination == DESTINATION_A {
                    agent.1.angle = (pos.y - t.y).atan2(pos.x - t.x);
                }
                let mut handle = rings.get_mut(agent.1.ring).unwrap();
                *handle = colors.add(ColorMaterial::from(Color::YELLOW));
            }

            if agent.1.counter_b > counter_b {
                agent.1.counter_b = counter_b + 350;
                
                if agent.1.trigger_timer <= 0.0 {
                    agent.1.trigger = true;
                    agent.1.trigger_timer = 0.5;
                }

                if agent.1.destination == DESTINATION_B {
                    agent.1.angle = (pos.y - t.y).atan2(pos.x - t.x);
                }
                let mut handle = rings.get_mut(agent.1.ring).unwrap();
                *handle = colors.add(ColorMaterial::from(Color::GREEN));
            }
        }
    }

    for (entity, mut agent, mut transform) in agents.iter_mut() {
        agent.counter_a += 1;
        agent.counter_b += 1;

        transform.translation.x += agent.angle.cos() * agent.speed * time.delta_seconds();
        transform.translation.y += agent.angle.sin() * agent.speed * time.delta_seconds();

        if transform.translation.x > 700.0 {agent.angle = std::f32::consts::PI - agent.angle;}
        if transform.translation.x < -700.0 {agent.angle = std::f32::consts::PI - agent.angle;}
        if transform.translation.y > 300.0 {agent.angle = -agent.angle;}
        if transform.translation.y < -300.0 {agent.angle = -agent.angle;}

        agent.angle += rand::thread_rng().gen_range(-0.02..=0.02);
        
        let mut ring = rings.get_mut(agent.ring).unwrap();

        let color = colors.get(&*ring).unwrap();
        let new_color = color.color.with_a((color.color.a() - 0.1).max(0.0));
        *ring = colors.add(ColorMaterial::from(new_color));
    }

}
