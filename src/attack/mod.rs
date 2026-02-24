pub mod bruteforce;
pub mod dictionary;

pub use bruteforce::bruteforce_attack;
pub use dictionary::{
    append_to_dictionary, dictionary_attack, ensure_dictionary_exists, get_default_dictionary_path,
};
