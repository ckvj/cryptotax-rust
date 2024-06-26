mod funcs;
use std::env;
use std::error::Error;
use colored::Colorize;
use funcs::process_trades::SaleEvent;
use std::path::PathBuf;
use polars::prelude::*;
use std::fs::File;



#[allow(warnings, dead_code)]

/// Given filepath to config file
fn main() -> Result<(), Box<dyn Error>> {

    // Config
    let config_filepath: PathBuf = collect_config_filepath()?;
    let config = match funcs::config::build_config(config_filepath) {
        Ok(config) => config,
        Err(ConfigParseError) => panic!("{}", ConfigParseError.to_string().on_purple()),
    };

    // Import Trades
    let trades = funcs::import_trades::import_trades(&config).unwrap();

    // Process Trades
    let (sale_events, cost_bases) = funcs::process_trades::get_sale_events_and_cost_basis(trades, &config);
    let mut annual_summary = funcs::process_trades::get_annual_summary(&sale_events);
    
    
    // Export
    println!("{:?}", cost_bases);
    println!("{}", annual_summary);
    vec_to_csv(&sale_events, "sale_events");
    df_to_csv(&annual_summary, "annual_summary")?;

    Ok(())
}


fn collect_config_filepath() -> Result<PathBuf, String> {
    match env::args().nth(1) {
        Some(filepath) => Ok(PathBuf::from(filepath)), // TODO: Check if file exists
        None => Err("Please provide a filepath to the config file as an argument".to_string()),
    }
}

fn df_to_csv(df: &DataFrame, csv_name: &str) -> Result<(), Box<dyn Error>> {
    let mut df = df.clone();
    let output_file: File = File::create(format!("{}.csv",csv_name))?;
    let mut writer: CsvWriter<File> = CsvWriter::new(output_file).include_header(true);
    writer.finish(&mut df)?;
    Ok(())
}

fn vec_to_csv(vec: &[SaleEvent], csv_name: &str) -> Result<(), Box<dyn Error>> {
    let vec = vec.to_owned();
    let mut writer = csv::Writer::from_path(format!("{}.csv",csv_name))?;
    for row in &vec {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}


// OLD CODE

  // let df = funcs::process_trades::convert_vec_to_df(&sale_events);

    // let grouped_df = df
    //     .group_by(["Asset Name", "Sell Year"])?
    //     .select(["Gain-Loss"]).sum().unwrap()
    //     .sort(&["Asset Name", "Sell Year"], false, false)?;
