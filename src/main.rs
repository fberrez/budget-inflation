use std::io;

use plotters::prelude::*;
use rand_distr::{Distribution, Normal};

fn simulate_inflation(
    years: usize,
    start_rate: f64,
    volatility: f64,
    mean_reversion: f64,
    long_term_mean: f64,
) -> Vec<f64> {
    let mut rates = vec![start_rate];
    let normal = Normal::new(0.0, volatility).unwrap();
    let mut rng = rand::thread_rng();

    for _ in 1..years {
        let drift = mean_reversion * (long_term_mean - rates.last().unwrap());
        let random_shock = normal.sample(&mut rng);
        let new_rate = (rates.last().unwrap() + drift + random_shock).max(0.0);
        rates.push(new_rate);
    }
    rates
}

fn run_multiple_simulations(
    num_simulations: usize,
    years: usize,
    start_rate: f64,
    volatility: f64,
    mean_reversion: f64,
    long_term_mean: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut all_simulations = vec![vec![0.0; years]; num_simulations];

    for sim in all_simulations.iter_mut() {
        *sim = simulate_inflation(
            years,
            start_rate,
            volatility,
            mean_reversion,
            long_term_mean,
        );
    }

    let mut mean_rates = vec![0.0; years];
    let mut lower_bound = vec![0.0; years];
    let mut upper_bound = vec![0.0; years];

    for year in 0..years {
        let mut year_rates: Vec<f64> = all_simulations.iter().map(|sim| sim[year]).collect();
        year_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());

        mean_rates[year] = year_rates.iter().sum::<f64>() / num_simulations as f64;
        lower_bound[year] = year_rates[num_simulations / 10];
        upper_bound[year] = year_rates[num_simulations * 9 / 10];
    }

    (mean_rates, lower_bound, upper_bound)
}

fn calculate_monthly_savings(
    goal: f64,
    years: usize,
    inflation_rates: &[f64],
    annual_return: f64,
) -> f64 {
    let mut future_goal = goal;
    for &rate in inflation_rates.iter().rev() {
        future_goal /= 1.0 + rate;
    }

    let monthly_return = (1.0 + annual_return).powf(1.0 / 12.0) - 1.0;
    let months = years * 12;

    (future_goal * monthly_return) / ((1.0 + monthly_return).powi(months as i32) - 1.0)
}

fn get_user_input<T: std::str::FromStr>(prompt: &str) -> T {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let goal: f64 = get_user_input("Enter your savings goal (in euros):");
    let current_age: u32 = get_user_input("Enter your current age:");
    let target_age: u32 = get_user_input("Enter your target age:");
    let monthly_salary: f64 = get_user_input("Enter your monthly net salary (in euros):");

    let years_to_simulate = (target_age - current_age) as usize;
    let start_inflation_rate = 0.02;
    let inflation_volatility = 0.005;
    let mean_reversion_speed = 0.3;
    let long_term_inflation_mean = 0.02;
    let num_simulations = 1000;
    let annual_return = 0.05; // Assuming a 5% annual return on investments

    let (mean_rates, lower_bound, upper_bound) = run_multiple_simulations(
        num_simulations,
        years_to_simulate,
        start_inflation_rate,
        inflation_volatility,
        mean_reversion_speed,
        long_term_inflation_mean,
    );

    let mean_savings =
        calculate_monthly_savings(goal, years_to_simulate, &mean_rates, annual_return);
    let lower_savings =
        calculate_monthly_savings(goal, years_to_simulate, &lower_bound, annual_return);
    let upper_savings =
        calculate_monthly_savings(goal, years_to_simulate, &upper_bound, annual_return);

    println!(
        "\nTo reach your goal of €{:.2} by age {}:",
        goal, target_age
    );
    println!("Based on mean inflation: €{:.2} per month", mean_savings);
    println!(
        "Based on lower bound inflation: €{:.2} per month",
        lower_savings
    );
    println!(
        "Based on upper bound inflation: €{:.2} per month",
        upper_savings
    );

    let savings_ratio = mean_savings / monthly_salary;
    println!(
        "\nThis represents {:.1}% of your current monthly salary.",
        savings_ratio * 100.0
    );

    // Plotting
    let root =
        BitMapBackend::new("france_inflation_simulation.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Simulated Inflation Rates in France",
            ("sans-serif", 30).into_font(),
        )
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..10.0, 0.0..0.04)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (0..years_to_simulate).map(|x| (x as f64, mean_rates[x])),
            &RED,
        ))?
        .label("Mean Inflation Rate")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .draw_series(AreaSeries::new(
            (0..years_to_simulate).map(|x| (x as f64, lower_bound[x])),
            0.0,
            &BLUE.mix(0.2),
        ))?
        .label("80% Confidence Interval")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE.mix(0.2)));

    chart.draw_series(AreaSeries::new(
        (0..years_to_simulate).map(|x| (x as f64, upper_bound[x])),
        0.0,
        &BLUE.mix(0.2),
    ))?;

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    println!(
        "Estimated inflation rates for the next {} years:",
        years_to_simulate
    );
    for (year, rate) in mean_rates.iter().enumerate() {
        println!("Year {}: {:.2}%", year + 1, rate * 100.0);
    }

    Ok(())
}
