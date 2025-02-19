pub mod utils;
mod get_linked_account_command;
mod uptime_command;
mod color_command;

pub fn get_all_commands() -> Vec<poise::Command<crate::Data, crate::types::Error>> {
    vec![color_command::color(), get_linked_account_command::get_linked_account(), uptime_command::uptime()]
}