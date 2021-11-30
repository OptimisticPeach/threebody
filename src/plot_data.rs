use plotters::prelude::*;
use plotters_backend::BackendColor;

struct LightGreen;

impl Color for LightGreen {
    fn to_backend_color(&self) -> BackendColor {
        BackendColor {
            rgb: (0, 255, 0),
            alpha: 0.4,
        }
    }
}

pub fn plot_distr(data: Vec<(f64, usize)>, name_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("results/plot_{}.png", name_id);
    let root = BitMapBackend::new(&filename, (1200, 1200)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut minx = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut miny = usize::MAX;
    let mut maxy = 0;

    data
        .iter()
        .for_each(|&(x, y)| {
            minx = minx.min(x);
            maxx = maxx.max(x);
            miny = miny.min(y);
            maxy = maxy.max(y);
        });

    let miny = miny as f64;
    let maxy = maxy as f64;

    let rangex = minx * 0.9..maxx * 1.1;
    let rangey = miny * 0.9..maxy * 1.1;

    let stepx = (maxx - minx) / 100.0;
    let stepy = (maxy - miny) / 100.0;

    let areas = root.split_by_breakpoints([1080], [120]);

    let mut x_hist_ctx = ChartBuilder::on(&areas[0])
        .y_label_area_size(40)
        .build_cartesian_2d(rangex.step(stepx).use_round().into_segmented(), 0..250)?;
    let mut y_hist_ctx = ChartBuilder::on(&areas[3])
        .x_label_area_size(40)
        .build_cartesian_2d(0..250, rangey.step(stepy).use_round())?;

    let mut scatter_ctx = ChartBuilder::on(&root)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(minx * 0.9..maxx * 1.1, miny * 0.9..maxy * 1.1)?;
    scatter_ctx
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;
    scatter_ctx.draw_series(
        data
            .iter()
            .map(|(x, y)| Circle::new((*x, *y as f64), 2.0, GREEN)),
    )?;

    let x_hist = Histogram::vertical(&x_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(data.iter().map(|(x, _)| (*x, 1)));
    let y_hist = Histogram::horizontal(&y_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(data.iter().map(|(_, y)| (*y as f64, 1)));
    x_hist_ctx.draw_series(x_hist)?;
    y_hist_ctx.draw_series(y_hist)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", filename);

    Ok(())
}
