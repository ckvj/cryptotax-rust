
/// All trades are classified as Buy, Sale, or Other, where Other can includes ignored, non-capital gains events (eg transfer)
#[derive(Debug, Clone, PartialEq)]
pub enum TxnType {
    Buy,
    Sale,
    Other,
}

impl TxnType {
    /// Returns txn_type enum for provided txn_type string and available classification vectors
    pub fn return_txn_type(
        match_string: &String,
        buy_vector: &[String],
        sell_vector: &[String],
    ) -> TxnType {
        match match_string {
            _ if contains_in_vector(&match_string, buy_vector) => TxnType::Buy, //Question: Why does analyzer recommend I remove the & in the &match_string?
            _ if contains_in_vector(&match_string, sell_vector) => TxnType::Sale,
            _ => TxnType::Other,
        }
    }
}


/// Returns bool of input string contained in any string in vector of strings
pub fn contains_in_vector(string: &str, string_vector: &[String]) -> bool {
    string_vector.
    iter().
    any(|s| string.contains(s))
}

