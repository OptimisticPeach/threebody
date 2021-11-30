mod plot_data;
mod run_simulations;

use glam::DVec3;
use rand::Rng;

use run_simulations::{StartingCondition, run_simulations};
use crate::plot_data::plot_distr;

lazy_static::lazy_static! {
    static ref DYN_STARTING_CONDS: [StartingCondition; 7] = [
        StartingCondition {
            pos: DVec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        },
        StartingCondition {
            pos: DVec3::new(-1.0, 3.0, -2.0),
            ..Default::default()
        },
        StartingCondition {
            pos: DVec3::new(2.0, -3.0, 1.0),
            ..Default::default()
        },
        StartingCondition {
            pos: DVec3::new(-3.0, -2.0, 2.0),
            ..Default::default()
        },
        StartingCondition {
            pos: DVec3::new(-2.0, 1.0, -3.0),
            ..Default::default()
        },
        StartingCondition {
            pos: DVec3::ZERO,
            ..Default::default()
        },
        StartingCondition {
            pos: DVec3::new(-4.0, 2.0, -3.0),
            ..Default::default()
        }
    ];
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = run_simulations(&DYN_STARTING_CONDS,
        (0..8000)
            .map(|_| {
                let mut rand = rand::thread_rng();
                move |_| {
                    let disp = rand.gen::<(f64, f64, f64)>();
                    DVec3::new(disp.0, disp.1, disp.2) * 0.01
                }
            })
    );

    let deviations = data.iter().map(|&(x, _)| x).collect::<Vec<_>>();

    println!("{:.5?}", deviations);

    let iterations = data.iter().map(|&(_, x)| x).collect::<Vec<_>>();

    println!("{:?}", iterations);

    plot_distr(data, format!("plus_one"))?;

    Ok(())
}
