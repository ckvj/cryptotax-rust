use cryptotax::funcs;
use cryptotax::funcs::config::AccountingType;

#[test]
fn integration_test() {

    let config_filepath = String::from("tests/test_config.ini");
    let mut config = funcs::config::build_config(&config_filepath).unwrap();

    assert!(matches!(config.accounting_type, AccountingType::FIFO | AccountingType::LIFO | AccountingType::HIFO));
    
    for i in [AccountingType::FIFO, AccountingType::LIFO, AccountingType::HIFO].iter() {
        config.accounting_type = i.clone();
        let trades = funcs::import_trades::import_trades(&config).unwrap();
        let sale_events = funcs::process_trades::get_sale_events(trades, &config);
        let gain_loss_summary: f32 = sale_events.iter().map(|sale_event| sale_event.gain_loss).sum();

        match config.accounting_type {
            AccountingType::FIFO => assert_eq!(format!("{:.2}", gain_loss_summary), format!("{:.2}", -2577.7231)),
            AccountingType::LIFO => assert_eq!(format!("{:.2}", gain_loss_summary), format!("{:.2}", -5249.9736)),
            AccountingType::HIFO => assert_eq!(format!("{:.2}", gain_loss_summary), format!("{:.2}", -5754.5923)),
        }
    }
}