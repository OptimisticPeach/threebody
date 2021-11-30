use glam::DVec3;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

#[derive(Copy, Clone, Debug)]
struct World<const N: usize> {
    forces: [DVec3; N],
    velocities: [DVec3; N],
    accelerations: [DVec3; N],
    positions: [DVec3; N],
    starting_deviation: f64,
}

const RADIUS: f64 = 0.1;
const TIMESTEP: f64 = 1.0 / (2 << 20) as f64;
const GRAV_CONST: f64 = 1.0;
const PER_ITER: usize = 1024;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct StartingCondition {
    pub pos: DVec3,
    pub mass: f64,
    pub vel: DVec3,
    pub rad: f64,
}

impl Default for StartingCondition {
    fn default() -> Self {
        Self {
            pos: DVec3::ZERO,
            mass: 50.0,
            vel: DVec3::ZERO,
            rad: RADIUS,
        }
    }
}

pub fn run_simulations<'a, I: 'a, PerInst: 'a, const N: usize>(starting_conditions: &[StartingCondition; N], instances: I) -> Vec<(f64, usize)>
where
    I: Iterator<Item = PerInst>,
    PerInst: FnMut(usize) -> DVec3 {
    let (mut first_world, mut other_worlds, masses, radii) = create_worlds(starting_conditions, instances);

    let mut world_count = other_worlds.len();

    let mut datapoints = Vec::new();

    let mut timesteps = 0;

    while other_worlds.len() > 0 {
        timesteps += PER_ITER;

        other_worlds
            .par_iter_mut()
            .chain(rayon::iter::once(&mut first_world))
            .for_each(|world| {
                for _ in 0..PER_ITER {
                    perform_calculations(&mut world.forces, &masses, &radii, &mut world.positions, &mut world.velocities, &mut world.accelerations);
                }
            });

        for i in (0..other_worlds.len()).rev() {
            let new_deviation = deviation_between(&first_world, &other_worlds[i]);

            // println!("{}, {}", new_deviation, initial_deviation);

            if new_deviation / other_worlds[i].starting_deviation >= 2.0 {
                println!("Killed {}", world_count);
                world_count -= 1;
                let initial_deviation = other_worlds.swap_remove(i).starting_deviation;
                datapoints.push((initial_deviation, timesteps));
            }
        }
    }

    datapoints
}

fn deviation_between<const N: usize>(world_1: &World<N>, world_2: &World<N>) -> f64 {
    world_1
        .positions
        .iter()
        .zip(world_2.positions.iter())
        .map(|(a, b)| *a - *b)
        .fold(0.0, |acc, delta| acc + delta.length())
}

fn create_worlds<'a, I: 'a, PerInst: 'a, const N: usize>(conds: &[StartingCondition; N], instances: I) -> (World<N>, Vec<World<N>>, [f64; N], [f64; N])
where
    I: Iterator<Item=PerInst>,
    PerInst: FnMut(usize) -> DVec3 {
    let first_world = World {
        positions: conds.clone().map(|x| x.pos),
        velocities: conds.clone().map(|x| x.vel),
        accelerations: [(); N].map(|_| DVec3::ZERO),
        forces: [(); N].map(|_| DVec3::ZERO),
        starting_deviation: 0.0,
    };

    let others = instances
        .map(|mut f| {
            let mut world = first_world.clone();

            let mut deviation = 0.0;

            world
                .positions
                .iter_mut()
                .enumerate()
                .for_each(|(idx, pos)| {
                    let displ = f(idx);
                    deviation += displ.length();
                    *pos += displ;
                });

            world.starting_deviation = deviation;

            world
        })
        .collect::<Vec<_>>();

    (first_world, others, conds.clone().map(|x| x.mass), conds.map(|x| x.rad))
}

fn perform_calculations<const N: usize>(
    forces: &mut [DVec3; N],
    masses: &[f64; N],
    radii: &[f64; N],
    positions: &mut [DVec3; N],
    velocities: &mut [DVec3; N],
    accelerations: &mut [DVec3; N],
) {
    // Calculate forces
    for i in 0..forces.len() {
        let my_mass = masses[i];
        let my_radius = radii[i];
        let my_pos = positions[i];

        let sum = (0..forces.len())
            .fold(
                DVec3::ZERO,
                |accumulation, index| {
                    let mass = masses[index];
                    let radius = radii[index];
                    let pos = positions[index];

                    let min_radius = (radius + my_radius) * (radius + my_radius);
                    let dir = pos - my_pos;
                    let len = dir.length_squared();
                    if len <= min_radius {
                        return accumulation;
                    }

                    let force_mag = GRAV_CONST * mass * my_mass / len;
                    accumulation + (force_mag * dir / len.sqrt())
                }
            );

        forces[i] = sum;
    }

    // Then integrate velocity then position.
    let dt = TIMESTEP;

    velocities
        .iter_mut()
        .zip(positions.iter_mut())
        .zip(accelerations.iter_mut())
        .zip(masses.iter())
        .zip(forces.iter())
        .for_each(|((((vel, pos), acc), mass), force)| {
            *acc = *force / *mass;
            *vel += *acc * dt;
            *pos += *vel * dt;
        });
}
