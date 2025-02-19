mod uptime_command;
pub mod utils;
mod color_command;
mod get_linked_account_command;

pub fn get_all_commands() -> Vec<poise::Command<crate::Data, crate::types::Error>> {
    vec![color_command::color(), get_linked_account_command::get_linked_account(), uptime_command::uptime()]
}